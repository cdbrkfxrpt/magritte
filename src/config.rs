// Cloneright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use clap::Parser;
use eyre::Result;
use serde::Deserialize;
use std::fs;


#[derive(Clone, Debug, Deserialize)]
pub struct Config {
  #[serde(skip)]
  pub command_line_args:    CommandLineArgs,
  pub database_credentials: DatabaseCredentials,
  pub channel_capacities:   ChannelCapacities,
  pub feeder_config:        FeederConfig,
}

impl Config {
  pub fn new() -> Result<Self> {
    let args = CommandLineArgs::parse();
    let mut conf: Self =
      toml::from_str(&fs::read_to_string(args.config_path.clone())?)?;

    conf.command_line_args = args;
    Ok(conf)
  }
}


#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseCredentials {
  pub host:     String,
  pub user:     String,
  pub password: String,
  pub dbname:   String,
}


#[derive(Clone, Debug, Deserialize)]
pub struct ChannelCapacities {
  pub inner:   usize,
  pub data:    usize,
  pub request: usize,
  pub source:  usize,
  pub sink:    usize,
}


#[derive(Clone, Debug, Deserialize)]
pub struct FeederConfig {
  pub millis_per_cycle:  u64,
  pub datapoints_to_run: usize,
  pub query:             FeederQuery,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FeederQuery {
  pub source_id:   String,
  pub timestamp:   String,
  pub value_names: Vec<String>,
  pub from_table:  String,
  pub order_by:    String,
}


/// Uses the `clap` crate to parse runtime parameters from the command line
#[derive(Parser, Clone, Debug, Default)]
#[clap(author, version, about)]
pub struct CommandLineArgs {
  /// Set path for config file
  #[clap(short, long, default_value = "./conf/magritte.toml")]
  pub config_path: String,
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod test {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn config_test() {
    let conf = CommandLineArgs::parse();

    assert_eq!(conf.config_path, String::from("./conf/magritte.toml"));
  }
}
