// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use clap::Parser;


/// Uses the `clap` crate to parse runtime configuration parameters from the
/// command line. These include things like execution step size, timeouts, ...
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Config {
  /// Set milliseconds per execution step
  #[clap(short, long, default_value = "1000")]
  pub millis_per_step: usize,
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod test {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn config_test() {
    let conf = Config::parse();

    assert_eq!(conf.millis_per_step, 1000);
  }
}
