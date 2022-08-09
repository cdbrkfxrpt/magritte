// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{database::DatabaseConnector, util};
use crate::services::{Broker, Sink, Source};

use clap::Parser;
use eyre::Result;
use serde::Deserialize;
use std::fs;
use tokio::time;
use tracing::info;


#[derive(Debug, Deserialize)]
/// Deserialized from config file. Initializes core elements of `magritte`.
pub struct AppCore {
  pub database_connector: DatabaseConnector,
  pub source:             Source,
  pub broker:             Broker,
  sink:                   Sink,
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

  pub async fn prepare_run(&self) -> Result<()> {
    self.database_connector.prepare_run().await

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
  // use pretty_assertions::assert_eq;


  #[test]
  fn app_core_test() {}
}
