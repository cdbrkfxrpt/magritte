// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use clap::Parser;


#[derive(Parser, Clone, Debug, Default, PartialEq)]
#[clap(author, version, about)]
/// Uses the `clap` crate to parse runtime parameters from the command line.
pub struct CommandLineArgs {
  /// Set path for config file
  #[clap(short, long, default_value = "./conf/magritte.toml")]
  pub config_path: String,
}


#[macro_export]
/// Use this like you would use `vec![]`, giving it `&str` elements as
/// arguments, and you'll get a `Vec<String>` with your elements.
macro_rules! stringvec {
  [$($x:literal),* $(,)?] => (vec![$(String::from($x)),*]);
}

#[allow(unused)]
pub fn round_f32(n: f32, d: i64) -> f32 {
  (n * (d * 10) as f32).round() / (d * 10) as f32
}

#[allow(unused)]
pub fn round_f64(n: f64, d: i64) -> f64 {
  (n * (d * 10) as f64).round() / (d * 10) as f64
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
