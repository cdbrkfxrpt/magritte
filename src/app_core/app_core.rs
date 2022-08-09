// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{database::DatabaseConnector, util};
use crate::services::{Broker, Sink, Source};

use clap::Parser;
use eyre::Result;
use indoc::indoc;
use serde::Deserialize;
use std::fs;
use tokio::time;
use tracing::info;


#[derive(Debug, Deserialize)]
/// Deserialized from config file. Initializes core elements of `magritte`.
pub struct AppCore {
  database_connector: DatabaseConnector,
  source:             Source,
  broker:             Broker,
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
  /// -- dropping and recreating magritte schema
  /// drop schema if exists magritte cascade;
  /// create schema magritte;
  ///
  /// -- creating event stream table
  /// create table magritte.event_stream (
  ///   id          serial,
  ///   fluent_name text,
  ///   keys        integer[],
  ///   timestamp   bigint,
  ///   value       bool,
  ///   lastChanged bigint
  /// );
  ///
  /// -- transforming coastline to the correct coordinate system:
  /// -- this is very costly but can be done only once on startup
  /// select gid, shape_leng, ST_Transform(geom, 3857) as geom
  /// into   magritte.europe_coastline
  /// from   geographic_features.europe_coastline;
  /// ```
  ///
  /// Furthermore, establishes the connections between `Broker`, `Source` and
  /// `Sink`, initializes the fluent nodes and registers them at the broker.
  pub async fn prepare_run(&self) -> Result<()> {
    let client = self.database_connector.connect().await?;

    let sql_raw = indoc! {r#"
      -- dropping and recreating magritte schema
      drop schema if exists magritte cascade;
      create schema magritte;

      -- creating event stream table
      create table magritte.event_stream (
        id          serial,
        fluent_name text,
        keys        integer[],
        timestamp   bigint,
        value       bool,
        lastChanged bigint
      );

      -- transforming coastline to the correct coordinate system:
      -- this is very costly but can be done only once on startup
      select gid, shape_leng, ST_Transform(geom, 3857) as geom
      into   magritte.europe_coastline
      from   geographic_features.europe_coastline;
    "#};

    info!("executing run preparation SQL:\n\n{}", sql_raw);
    client.batch_execute(sql_raw).await?;

    Ok(())

    // broker.register_source(source);
    // broker.register_sink(sink);
    // for node in build_node_index() {
    //   broker.register_node(node)?;
    // }
  }

  pub async fn run(self) -> Result<()> {
    let mut counter = 0;
    let mut interval = time::interval(time::Duration::from_millis(256));
    loop {
      info!("counter value at {:8}", counter);
      interval.tick().await;
      counter += 1;
    }
  }
}


// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::AppCore;
  use crate::{fluent::AnyFluent, stringvec};

  use pretty_assertions::assert_eq;
  use tokio::sync::mpsc;


  #[tokio::test]
  async fn source_test() {
    let app_core = AppCore::init().unwrap();

    let database_connector =
      app_core.database_connector.connect().await.unwrap();
    let source = app_core.source;

    assert_eq!(source.published_fluents(),
               stringvec!["lon", "lat", "speed"]);

    let (tx, mut rx) = mpsc::unbounded_channel();

    let runner = tokio::spawn(async move {
      source.run(database_connector, tx).await.unwrap();
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
  async fn app_core_test() {
    let app_core = AppCore::init().unwrap();

    assert!(app_core.prepare_run().await.is_ok());
  }
}
