// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::BrokerConfig, source::Source, types::Message};

use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tracing::info;


type Sources = Arc<Mutex<HashMap<usize, mpsc::Sender<Message>>>>;


#[derive(Debug)]
pub struct Broker {
  config:          BrokerConfig,
  request_tx:      mpsc::Sender<Message>,
  sources:         Sources,
  data_handler:    DataHandler,
  request_handler: MessageHandler,
}

impl Broker {
  pub fn init(config: BrokerConfig,
              feeder_rx: mpsc::Receiver<Message>,
              sink_tx: mpsc::Sender<Message>)
              -> Self {
    let (request_tx, request_rx) = mpsc::channel(config.channel_capacity);
    let sources = Arc::new(Mutex::new(HashMap::new()));

    let data_handler = DataHandler::init(feeder_rx,
                                         sink_tx,
                                         request_tx.clone(),
                                         sources.clone());

    let request_handler = MessageHandler::init(request_rx, sources.clone());

    Self { config,
           request_tx,
           sources,
           data_handler,
           request_handler }
  }

  pub fn run(self) {
    self.data_handler.run(self.config.channel_capacity);
    self.request_handler.run();
  }
}


#[derive(Debug)]
pub struct DataHandler {
  feeder_rx:  mpsc::Receiver<Message>,
  sink_tx:    mpsc::Sender<Message>,
  request_tx: mpsc::Sender<Message>,
  sources:    Sources,
}

impl DataHandler {
  pub fn init(feeder_rx: mpsc::Receiver<Message>,
              sink_tx: mpsc::Sender<Message>,
              request_tx: mpsc::Sender<Message>,
              sources: Sources)
              -> Self {
    Self { feeder_rx,
           sink_tx,
           request_tx,
           sources }
  }

  pub fn run(mut self, channel_capacity: usize) {
    tokio::spawn(async move {
      // source spawning and data sending task
      while let Some(message) = self.feeder_rx.recv().await {
        // unwrapping Mutex lock is safe per Mutex docs
        let mut sources = self.sources.lock().await;

        let Message::Datapoint { source_id, .. } = message else {
          panic!("data sent from Feeder not a datapoint");
        };

        let source_tx = if !sources.contains_key(&source_id) {
          let (source_tx, source_rx) = mpsc::channel(channel_capacity);

          sources.insert(source_id, source_tx.clone());
          Source::init(source_id,
                       source_rx,
                       self.request_tx.clone(),
                       self.sink_tx.clone()).run();

          source_tx
        } else {
          sources[&source_id].clone()
        };

        source_tx.send(message).await.unwrap();
      }
    });
  }
}


#[derive(Debug)]
pub struct MessageHandler {
  request_rx: mpsc::Receiver<Message>,
  sources:    Sources,
}

impl MessageHandler {
  pub fn init(request_rx: mpsc::Receiver<Message>, sources: Sources) -> Self {
    Self { request_rx,
           sources }
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      while let Some(message) = self.request_rx.recv().await {
        info!(?message);
      }
    });
  }
}
