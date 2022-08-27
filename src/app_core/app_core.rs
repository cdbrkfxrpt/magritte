// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{broker::Broker,
            database::Database,
            sink::Sink,
            source::Source,
            util};
use crate::{fluent::{Fluent, FluentTrait, ValueType},
            handler::{EvalFn, Handler, HandlerDefinition, KeyDependency}};

use clap::Parser;
use eyre::Result;
use futures::future::FutureExt;
use indoc::indoc;
use serde::Deserialize;
use std::{fs, sync::Arc};
use tracing::{debug, info};


#[derive(Debug, Deserialize)]
/// Deserialized from config file. Initializes core elements of `magritte`.
/// From here, all core elements and fluent [`Handler`]s are initialized and
/// put into operation via the `run` method.
pub struct AppCore {
  database:       Database,
  broker:         Broker,
  source:         Source,
  sinks:          Vec<Sink>,
  buffer_timeout: usize,
}

impl AppCore {
  /// Method does not require parameters; options are taken from command line,
  /// parameters are parsed from a (required) config file.
  pub fn init() -> Result<Self> {
    let args = util::CommandLineArgs::parse();
    let app_init: Self =
      toml::from_str(&fs::read_to_string(args.config_path.clone())?)?;

    Ok(app_init)
  }

  /// Prepares the database for a run using the following PostgreSQL:
  ///
  /// ```sql
  #[doc = include_str!("./sql/prepare_run.sql")]
  /// ```
  ///
  /// Furthermore, registers the [`Source`], [`Sink`] and fluent nodes at the
  /// broker and makes `magritte` ready to run. Finally - runs the application.
  /// Consumes the `AppCore` object.
  pub async fn run(self) -> Result<()> {
    // decompose self into contained handles
    let Self { database,
               mut broker,
               mut source,
               mut sinks,
               buffer_timeout, } = self;

    // run prep
    let client = database.connect().await?;

    let sql_raw = include_str!("./sql/prepare_run.sql");
    debug!("executing SYSTEM run preparation SQL:\n\n{}", sql_raw);
    client.batch_execute(sql_raw).await?;

    let sql_raw = include_str!("../../conf/prepare_run.sql");
    debug!("executing USER run preparation SQL:\n\n{}", sql_raw);
    client.batch_execute(sql_raw).await?;

    broker.register(&mut source);
    for sink in sinks.iter_mut() {
      broker.register(sink);
    }

    // initialize and run nodes
    let mut node_tasks = Vec::new();
    for def in include!("../../conf/handler_definitions.rs") {
      let mut node =
        Handler::new(def, buffer_timeout, database.clone()).await?;

      broker.register(&mut node);

      let task = tokio::spawn(async move {
        node.run().await.expect("Node has stopped");
      });
      node_tasks.push(task);
    }

    // establish a database connection and create a database client, then start
    // the sink task, creating a handle to the task.
    info!("starting sink tasks...");
    let mut sink_tasks = Vec::new();
    for sink in sinks {
      let sink_dbc = database.connect().await?;
      let sink_task = tokio::spawn(async move {
        sink.run(Some(sink_dbc)).await.expect("sink has stopped");
      });
      sink_tasks.push(sink_task);
    }

    // establish a database connection and create a database client, then start
    // the source task, creating a handle to the task.
    info!("starting source task...");
    let source_dbc = database.connect().await?;
    let source_task = tokio::spawn(async move {
      source.run(Some(source_dbc))
            .await
            .expect("source has stopped");
    });

    // start the broker task, creating a handle to the task.
    info!("starting broker...");
    broker.run().await?;

    // if broker task has ended, abort all tasks.
    info!("broker stopped, aborting all tasks...");
    for node_task in node_tasks {
      node_task.abort();
    }
    for sink_task in sink_tasks {
      sink_task.abort();
    }
    source_task.abort();

    Ok(())
  }
}


mod usr {
  use super::ValueType;


  pub fn return_value<T: ValueType>(value: T) -> Option<Box<dyn ValueType>> {
    Some(Box::new(value) as Box<dyn ValueType>)
  }
}


// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::AppCore;
  use crate::{app_core::node::{Node, NodeRx},
              fluent::FluentTrait,
              stringvec};

  use pretty_assertions::assert_eq;
  use tokio::sync::mpsc;


  #[tokio::test]
  async fn source_test() {
    let app_core = AppCore::init().unwrap();

    let mut source = app_core.source;

    assert_eq!(source.publishes(), stringvec!["lon", "lat", "speed"]);
    assert_eq!(source.subscribes_to(), Vec::<String>::new());

    let (tx, mut rx) = mpsc::unbounded_channel();
    source.initialize(tx, NodeRx::new());

    let database_client = app_core.database.connect().await.unwrap();
    let runner = tokio::spawn(async move {
      source.run(Some(database_client)).await.unwrap();
    });

    // the first value is the instant for time measurement; we need to skip
    // that and take the first "real" value instaed.
    rx.recv().await.unwrap();
    let fluent = rx.recv().await.unwrap();

    assert_eq!(fluent.name(), "lon");
    assert_eq!(fluent.keys(), &[245257000]);
    assert_eq!(fluent.timestamp(), 1443650402);
    assert_eq!(fluent.value::<f64>(), -4.4657183);
    assert_eq!(fluent.last_change(), 1443650402);

    runner.abort();
  }

  #[tokio::test]
  async fn source_unitialized_test() {
    let app_core = AppCore::init().unwrap();

    let source = app_core.source;
    let database_client = app_core.database.connect().await.unwrap();

    assert_eq!(source.publishes(), stringvec!["lon", "lat", "speed"]);
    assert_eq!(source.subscribes_to(), Vec::<String>::new());
    assert!(source.run(Some(database_client)).await.is_err());
  }
}
