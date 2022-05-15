// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::SourceBrokerConfig, datapoint::DataPoint};

use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tracing::info;


type SourcesIndex = Arc<Mutex<HashMap<usize, mpsc::Sender<DataPoint>>>>;


#[derive(Debug)]
pub struct SourceBroker {
  config:        SourceBrokerConfig,
  // collector_tx: Sender<EvalResult>,
  // globalbroker_tx: Sender<GlobalBrokerRequest>,
  feeder_rx:     mpsc::Receiver<DataPoint>,
  request_tx:    mpsc::Sender<SourceBrokerRequest>,
  request_rx:    mpsc::Receiver<SourceBrokerRequest>,
  sources_index: SourcesIndex,
}

impl SourceBroker {
  pub fn init(config: SourceBrokerConfig,
              // collector_tx: mpsc::Sender<EvalResult>,
              // globalbroker_tx: mpsc::Sender<GlobalBrokerReq>,
              feeder_rx: mpsc::Receiver<DataPoint>)
              -> Self {
    let (request_tx, request_rx) = mpsc::channel(config.channel_capacity);
    let sources_index = Arc::new(Mutex::new(HashMap::new()));

    Self { config,
           // collector_tx,
           // globalbroker_tx,
           request_tx,
           request_rx,
           feeder_rx,
           sources_index }
  }

  pub fn run(mut self) {
    let sources_index = self.sources_index.clone();

    tokio::spawn(async move {
      while let Some(datapoint) = self.feeder_rx.recv().await {
        // unwrapping Mutex lock is safe per Mutex docs
        let mut sources_index = sources_index.lock().await;

        let source_tx = if !sources_index.contains_key(&datapoint.source_id) {
          let (source_tx, mut source_rx) =
            mpsc::channel(self.config.channel_capacity);

          sources_index.insert(datapoint.source_id, source_tx.clone());
          tokio::spawn(async move {
            while let Some(datapoint) = source_rx.recv().await {
              info!(?datapoint);
            }
          });

          source_tx
        } else {
          sources_index[&datapoint.source_id].clone()
        };

        source_tx.send(datapoint).await.unwrap();
      }
    });
  }
}


#[derive(Debug)]
pub struct SourceBrokerRequest {}
