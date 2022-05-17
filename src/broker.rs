// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::BrokerConfig,
            source::Source,
            types::{Datapoint, FluentResult, Message}};

use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tracing::info;


type Sources = Arc<Mutex<HashMap<usize, mpsc::Sender<Message>>>>;


#[derive(Debug)]
pub struct Broker {
  config:          BrokerConfig,
  message_tx:      mpsc::Sender<Message>,
  sources:         Sources,
  data_handler:    DataHandler,
  message_handler: MessageHandler,
}

impl Broker {
  pub fn init(config: BrokerConfig,
              feeder_rx: mpsc::Receiver<Datapoint>,
              sink_tx: mpsc::Sender<FluentResult>)
              -> Self {
    let (message_tx, message_rx) = mpsc::channel(config.channel_capacity);
    let sources = Arc::new(Mutex::new(HashMap::new()));

    let data_handler = DataHandler::init(feeder_rx,
                                         sink_tx,
                                         message_tx.clone(),
                                         sources.clone());

    let message_handler = MessageHandler::init(message_rx, sources.clone());

    Self { config,
           message_tx,
           sources,
           data_handler,
           message_handler }
  }

  pub fn run(self) {
    self.data_handler.run(self.config.channel_capacity);
    self.message_handler.run();
  }
}


#[derive(Debug)]
pub struct DataHandler {
  feeder_rx:  mpsc::Receiver<Datapoint>,
  sink_tx:    mpsc::Sender<FluentResult>,
  message_tx: mpsc::Sender<Message>,
  sources:    Sources,
}

impl DataHandler {
  pub fn init(feeder_rx: mpsc::Receiver<Datapoint>,
              sink_tx: mpsc::Sender<FluentResult>,
              message_tx: mpsc::Sender<Message>,
              sources: Sources)
              -> Self {
    Self { feeder_rx,
           sink_tx,
           message_tx,
           sources }
  }

  pub fn run(mut self, channel_capacity: usize) {
    tokio::spawn(async move {
      // source spawning and data sending task
      while let Some(datapoint) = self.feeder_rx.recv().await {
        // unwrapping Mutex lock is safe per Mutex docs
        let mut sources = self.sources.lock().await;

        let source_tx = if !sources.contains_key(&datapoint.source_id) {
          let (source_tx, source_rx) = mpsc::channel(channel_capacity);

          sources.insert(datapoint.source_id, source_tx.clone());
          Source::init(datapoint.source_id,
                       source_rx,
                       self.message_tx.clone(),
                       self.sink_tx.clone()).run();

          source_tx
        } else {
          sources[&datapoint.source_id].clone()
        };

        source_tx.send(Message::Data(datapoint)).await.unwrap();
      }
    });
  }
}


#[derive(Debug)]
pub struct MessageHandler {
  message_rx: mpsc::Receiver<Message>,
  sources:    Sources,
}

impl MessageHandler {
  pub fn init(message_rx: mpsc::Receiver<Message>, sources: Sources) -> Self {
    Self { message_rx,
           sources }
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      while let Some(message) = self.message_rx.recv().await {
        info!(?message);
      }
    });
  }
}
