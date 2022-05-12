// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::config::Config;

use clap::Parser;
use tokio::task::JoinHandle;


#[derive(Debug)]
pub struct Magritte {
  config: Config,
}

impl Magritte {
  pub fn new() -> Self {
    let config = Config::parse();

    Self { config }
  }

  pub fn run(self) -> JoinHandle<()> {
    tokio::spawn(async move {})
  }
}
