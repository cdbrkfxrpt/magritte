// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::SourceBrokerConfig, datapoint::DataPoint};

use tokio::sync::mpsc;
use tracing::info;


#[derive(Debug)]
pub struct SourceBroker {
  config:    SourceBrokerConfig,
  // global_broker_tx: Sender<GlobalBrokerReq>,
  // collector_tx: Sender<EvalResult>,
  // sources_index: HashMap<usize, Sender<DataPoint>,
  feeder_rx: mpsc::Receiver<DataPoint>,
}

impl SourceBroker {
  pub fn init(config: SourceBrokerConfig,
              feeder_rx: mpsc::Receiver<DataPoint>)
              -> Self {
    Self { config, feeder_rx }
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      while let Some(datapoint) = self.feeder_rx.recv().await {
        info!(?datapoint);
      }
    });
  }
}
