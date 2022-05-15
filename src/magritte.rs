// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{cli::CommandLineArgs,
            config::Config,
            feeder::Feeder,
            sourcebroker::SourceBroker};

use clap::Parser;
use eyre::Result;
use std::fs;
use tokio::task::JoinHandle;
use tracing::{error, info};


#[derive(Debug)]
pub struct Magritte {
  config:       Config,
  feeder:       Feeder,
  sourcebroker: SourceBroker,
}

impl Magritte {
  pub fn new() -> Result<Self> {
    let config_path = CommandLineArgs::parse().path_to_config;
    let config: Config = toml::from_str(&fs::read_to_string(config_path)?)?;

    let (feeder, feeder_rx) = Feeder::init(config.feeder.clone());
    let sourcebroker =
      SourceBroker::init(config.sourcebroker.clone(), feeder_rx);

    Ok(Self { config,
              feeder,
              sourcebroker })
  }

  pub fn run(self) -> JoinHandle<()> {
    tokio::spawn(async move {
      self.sourcebroker.run();

      match self.feeder.run().await {
        Ok(()) => info!("Feeder has completed data"),
        Err(e) => error!("Feeder could not complete task: {}", e),
      }
    })
  }
}
