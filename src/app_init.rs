// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{broker::Broker, database::DatabaseConnector, source::Source};

use clap::Parser;
use eyre::Result;
use serde::Deserialize;
use std::fs;


#[derive(Parser, Clone, Debug, Default, PartialEq)]
#[clap(author, version, about)]
/// Uses the `clap` crate to parse runtime parameters from the command line.
pub struct CommandLineArgs {
  /// Set path for config file
  #[clap(short, long, default_value = "./conf/magritte.toml")]
  pub config_path: String,
}


#[derive(Debug, Deserialize)]
/// Deserialized from config file. Initializes core elements of `magritte`.
pub struct AppInit {
  pub database_connector: DatabaseConnector,
  pub source:             Source,
  pub broker:             Broker,
  // sink:               Sink,
}

impl AppInit {
  /// Method does not require parameters; options are taken from command line,
  /// parameters are parsed from a (required) config file.
  pub fn parse() -> Result<Self> {
    let args = CommandLineArgs::parse();
    let app_init: Self =
      toml::from_str(&fs::read_to_string(args.config_path.clone())?)?;

    Ok(app_init)
  }
}


// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::CommandLineArgs;

  use clap::Parser;
  use pretty_assertions::assert_eq;


  #[test]
  fn cla_test() {
    let cla = CommandLineArgs::parse();
    assert_eq!(cla.config_path, String::from("./conf/magritte.toml"));
  }

  // #[test]
  // fn app_init_test() {
  //   let app_init = AppInit::parse().unwrap();
  // }
}
