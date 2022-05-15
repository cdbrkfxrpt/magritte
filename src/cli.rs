// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use clap::Parser;


/// Uses the `clap` crate to parse runtime parameters from the command line
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct CommandLineArgs {
  /// Set capacity for task communication channels
  #[clap(short, long, default_value = "32")]
  pub channel_capacity: usize,
  /// Set milliseconds per execution cycle
  #[clap(short, long, default_value = "1000")]
  pub millis_per_cycle: usize,
  /// Set path for config file
  #[clap(short, long, default_value = "./conf/magritte.toml")]
  pub path_to_config:   String,
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod test {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn config_test() {
    let conf = CommandLineArgs::parse();

    assert_eq!(conf.channel_capacity, 32);
    assert_eq!(conf.millis_per_cycle, 1000);
    assert_eq!(conf.path_to_config, String::from("./conf/magritte.toml"));
  }
}
