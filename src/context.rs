// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::Config, evalresult::EvalResult};

use clap::Parser;
use eyre::Result;
use getset::Getters;
use tokio::sync::mpsc;


#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct Context {
  config:    Config,
  collector: (mpsc::Sender<EvalResult>, mpsc::Receiver<EvalResult>),
}

impl Context {
  pub fn new() -> Result<Self> {
    let config = Config::parse();

    Ok(Self { config,
              collector: mpsc::channel(config.channel_capacity) })
  }

  pub fn millis_per_cycle(&self) -> usize {
    self.config.millis_per_cycle
  }

  pub fn channel_capacity(&self) -> usize {
    self.config.channel_capacity
  }
}
