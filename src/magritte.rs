// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::Config, datapoint::DataPoint, feeder::Feeder};

use clap::Parser;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tracing::info;


#[derive(Debug)]
pub struct Magritte {
  config:      Config,
  feeder:      Feeder,
  from_feeder: Receiver<DataPoint>,
}

impl Magritte {
  pub fn new() -> Self {
    let config = Config::parse();

    let (feeder, from_feeder) = Feeder::init(&config);

    Self { config,
           feeder,
           from_feeder }
  }

  pub fn run(mut self) -> JoinHandle<()> {
    tokio::spawn(async move {
      self.feeder.run();

      while let Some(datapoint) = self.from_feeder.recv().await {
        info!(?datapoint);
      }
    })
  }
}
