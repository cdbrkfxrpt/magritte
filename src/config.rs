// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

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


#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Stores all configuration parameters for the application.
pub struct Config {
  #[serde(skip)]
  pub command_line_args: CommandLineArgs,
  pub database_params:   DatabaseParams,
  pub source_params:     SourceParams,
}

impl Config {
  /// Method does not require parameters; parameters are parsed from command
  /// line or from defaults, and from a config file.
  pub fn new() -> Result<Self> {
    let args = CommandLineArgs::parse();
    let mut conf: Self =
      toml::from_str(&fs::read_to_string(args.config_path.clone())?)?;

    conf.command_line_args = args;
    Ok(conf)
  }
}


#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Holds parameters to establish the connection to database.
pub struct DatabaseParams {
  pub host:     String,
  pub user:     String,
  pub password: String,
  pub dbname:   String,
}


#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Holds parameters for the `Source` service of the application, which reads
/// data from the source (i.e. the PostgreSQL database) and publishes it to the
/// `Broker` service.
pub struct SourceParams {
  pub run_params:   SourceRunParams,
  pub query_params: SourceQueryParams,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Holds parameters for the execution of the `Source` service.
pub struct SourceRunParams {
  pub millis_per_cycle:  u64,
  pub datapoints_to_run: usize,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Holds parameters for the database query performed by the `Source` service.
pub struct SourceQueryParams {
  pub key_name:       String,
  pub timestamp_name: String,
  pub fluent_names:   Vec<String>,
  pub from_table:     String,
  pub order_by:       String,
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::{CommandLineArgs,
              Config,
              DatabaseParams,
              SourceParams,
              SourceQueryParams,
              SourceRunParams};
  use crate::util::stringvec;

  use clap::Parser;
  use pretty_assertions::assert_eq;


  #[test]
  fn command_line_test() {
  }

  #[test]
  fn config_test() {
    let cla = CommandLineArgs::parse();
    assert_eq!(cla.config_path, String::from("./conf/magritte.toml"));

    let config = Config::new().unwrap();
    assert_eq!(config.command_line_args, cla);
    assert_eq!(config.command_line_args.config_path,
               String::from("./conf/magritte.toml"));
  }

  #[test]
  fn database_params_test() {
    let host = String::from("morpheus");
    let user = String::from("neo");
    let password = String::from("trinity");
    let dbname = String::from("nebukadnezar");

    let dbp = DatabaseParams { host:     host.clone(),
                               user:     user.clone(),
                               password: password.clone(),
                               dbname:   dbname.clone(), };

    assert_eq!(dbp.host, host);
    assert_eq!(dbp.user, user);
    assert_eq!(dbp.password, password);
    assert_eq!(dbp.dbname, dbname);
  }

  #[test]
  fn source_params_test() {
    let millis_per_cycle = 42;
    let datapoints_to_run = 1337;

    let srp = SourceRunParams { millis_per_cycle,
                                datapoints_to_run };

    assert_eq!(srp.millis_per_cycle, millis_per_cycle);
    assert_eq!(srp.datapoints_to_run, datapoints_to_run);

    let key_name = String::from("id");
    let timestamp_name = String::from("ts");
    let fluent_names = stringvec!["lat", "lon", "speed"];
    let from_table = String::from("the.matrix");
    let order_by = String::from("serial");

    let sqp = SourceQueryParams { key_name:       key_name.clone(),
                                  timestamp_name: timestamp_name.clone(),
                                  fluent_names:   fluent_names.clone(),
                                  from_table:     from_table.clone(),
                                  order_by:       order_by.clone(), };

    assert_eq!(sqp.key_name, key_name);
    assert_eq!(sqp.timestamp_name, timestamp_name);
    assert_eq!(sqp.fluent_names, fluent_names);
    assert_eq!(sqp.from_table, from_table);
    assert_eq!(sqp.order_by, order_by);

    let sp = SourceParams { run_params:   srp.clone(),
                            query_params: sqp.clone(), };

    assert_eq!(sp.run_params, srp);
    assert_eq!(sp.query_params, sqp);
  }
}
