// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use clap::Parser;


#[derive(Debug, Parser)]
#[clap(author, version, about)]
/// Uses the `clap` crate to parse runtime parameters from the command line.
pub struct CommandLineArgs {
  /// Set path for config file
  #[clap(short, long, default_value = "./conf/app_core.toml")]
  pub config_path: String,
}


#[macro_export]
/// Alias for `vec![]` that takes `&str`s and creates a `Vec<String>`.
macro_rules! stringvec {
  [$( $x:literal ),* $(,)?] => (vec![$( String::from($x) ),*]);
}


#[macro_export]
/// Alias for `vec![]` that takes `T`s and creates a `Vec<Box<T>>`.
macro_rules! boxvec {
  [$( $x:expr ),* $(,)?] => (vec![$( Box::new($x) ),*]);
}


#[macro_export]
/// Alias for `vec![]` that takes `&T`s and creates a `Vec<&(dyn ToSql +
/// Sync)>`. Bound for `T: ToSql + Sync`.
macro_rules! sqlvec {
  [$( $x:expr ),* $(,)?] => (vec![$( $x as &(dyn ToSql + Sync) ),*]);
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
    assert_eq!(cla.config_path, String::from("./conf/app_core.toml"));
  }
}
