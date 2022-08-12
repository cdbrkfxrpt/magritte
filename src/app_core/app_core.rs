// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{util, DatabaseConnector};
use crate::{boxvec,
            services::{Broker, Node, Sink, Source}};

use clap::Parser;
use eyre::Result;
use serde::Deserialize;
use std::fs;
use tracing::info;


#[derive(Debug, Deserialize)]
/// Deserialized from config file. Initializes core elements of `magritte`.
pub struct AppCore {
  database_connector: DatabaseConnector,
  broker:             Broker,
  source:             Source,
  sink:               Sink,
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
  #[doc = include_str!("prepare_run.sql")]
  /// ```
  ///
  /// Furthermore, registers the [`Source`], [`Sink`] and fluent nodes at the
  /// broker and makes `magritte` ready to run.
  pub async fn prepare_run(&mut self) -> Result<()> {
    let client = self.database_connector.connect().await?;

    let sql_raw = include_str!("prepare_run.sql");
    info!("executing SYSTEM run preparation SQL:\n\n{}", sql_raw);
    client.batch_execute(sql_raw).await?;

    let sql_raw = include_str!("../../conf/prepare_run.sql");
    info!("executing USER run preparation SQL:\n\n{}", sql_raw);
    client.batch_execute(sql_raw).await?;

    let nodes: Vec<Box<&mut dyn Node>> =
      boxvec![&mut self.source, &mut self.sink];

    // for node in build_node_index() {
    //   nodes.push(node);
    // }

    self.broker.register_all(nodes);
    Ok(())
  }

  pub async fn run(self) -> Result<()> {
    // in here we want to start
    // - the broker in its own worker task
    // - the sink in its own worker task
    // - the source, which  we're awaiting
    //
    tokio::spawn(async move {
      self.broker.run().await.expect("broker has stopped");
    });
    Ok(())
  }
}


// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::AppCore;
  use crate::{fluent::AnyFluent,
              services::{Node, NodeRx, StructuralNode},
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

    let database_client = app_core.database_connector.connect().await.unwrap();
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
    let database_client = app_core.database_connector.connect().await.unwrap();

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
