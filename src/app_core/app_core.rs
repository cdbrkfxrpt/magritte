// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{broker::Broker, database::Database, util};
use crate::{fluent::{AnyFluent, EvalFn},
            nodes::{FluentHandler, Node, Sink, Source}};

use boolinator::Boolinator;
use clap::Parser;
use eyre::Result;
use indoc::indoc;
use serde::Deserialize;
use std::fs;
use tracing::info;


#[derive(Debug, Deserialize)]
/// Deserialized from config file. Initializes core elements of `magritte`.
pub struct AppCore {
  database:        Database,
  broker:          Broker,
  source:          Box<Source>,
  sinks:           Vec<Box<Sink>>,
  #[serde(skip)]
  fluent_handlers: Vec<Box<FluentHandler>>,
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
  #[doc = include_str!("../sql/prepare_run.sql")]
  /// ```
  ///
  /// Furthermore, registers the [`Source`], [`Sink`] and fluent nodes at the
  /// broker and makes `magritte` ready to run.
  pub async fn prepare_run(&mut self) -> Result<()> {
    let client = self.database.connect().await?;

    let sql_raw = include_str!("../sql/prepare_run.sql");
    info!("executing SYSTEM run preparation SQL:\n\n{}", sql_raw);
    client.batch_execute(sql_raw).await?;

    let sql_raw = include_str!("../../conf/prepare_run.sql");
    info!("executing USER run preparation SQL:\n\n{}", sql_raw);
    client.batch_execute(sql_raw).await?;

    self.broker.register(&mut self.source);
    for sink in self.sinks.iter_mut() {
      self.broker.register(sink);
    }

    for def in include!("../../conf/fluent_handlers.rs") {
      let fluent_handler = FluentHandler::new(def.0, &def.1, def.2, def.3);
      self.fluent_handlers.push(Box::new(fluent_handler));
    }
    self.broker.register_nodes(&mut self.fluent_handlers);

    Ok(())
  }

  /// Runs the application. Consumes the `AppCore` object.
  pub async fn run(self) -> Result<()> {
    // decompose self into contained handles
    let Self { database,
               broker,
               source,
               sinks,
               fluent_handlers, } = self;

    // start nodes
    info!("starting node tasks...");
    let mut handler_tasks = Vec::new();
    for handler in fluent_handlers {
      // Boolinator makes life easy (and Booleans into Options)
      let handler_requires_dbc = handler.requires_dbc();
      let dbc = handler_requires_dbc.as_some(database.connect().await?);

      let handler_task = tokio::spawn(async move {
        handler.run(dbc).await.expect("node has stopped");
      });
      handler_tasks.push(handler_task);
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
    for handler_task in handler_tasks {
      handler_task.abort();
    }
    for sink_task in sink_tasks {
      sink_task.abort();
    }
    source_task.abort();

    Ok(())
  }
}


// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::AppCore;
  use crate::{fluent::AnyFluent,
              nodes::{Node, NodeRx, StructuralNode},
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
      source.run(database_client).await.unwrap();
    });

    let AnyFluent::FloatPt(fluent) = rx.recv().await.unwrap() else {
      panic!()
    };

    assert_eq!(fluent.name(), "lon");
    assert_eq!(fluent.keys(), &[245257000]);
    assert_eq!(fluent.timestamp(), 1443650402);
    assert_eq!(fluent.value(), &-4.4657183);
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
    assert!(source.run(database_client).await.is_err());
  }

  #[tokio::test]
  async fn app_core_test() {
    let mut app_core = AppCore::init().unwrap();

    assert!(app_core.prepare_run().await.is_ok());
  }
}
