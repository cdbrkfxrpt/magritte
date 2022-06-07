// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{fluent::{build_fluents_index, Fluent, FluentBase},
            types::{BrokerMessage, BrokerRequest, FluentResult}};

use std::collections::HashMap;
use tokio::sync::mpsc;


#[derive(Debug)]
pub struct Source {
  source_id:  usize,
  source_rx:  mpsc::Receiver<BrokerMessage>,
  request_tx: mpsc::Sender<BrokerRequest>,
  sink_tx:    mpsc::Sender<FluentResult>,
  fluents:    HashMap<String, mpsc::Sender<BrokerMessage>>,
}

impl Source {
  pub fn init(source_id: usize,
              source_rx: mpsc::Receiver<BrokerMessage>,
              request_tx: mpsc::Sender<BrokerRequest>,
              sink_tx: mpsc::Sender<FluentResult>)
              -> Self {
    let fluents = HashMap::new();

    Self { source_id,
           source_rx,
           request_tx,
           sink_tx,
           fluents }
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      // initialize and run Fluents
      for (name, fluent) in build_fluents_index() {
        // TODO capacity from config
        let (fluent_tx, fluent_rx) = mpsc::channel(32);
        self.fluents.insert(name.clone(), fluent_tx);

        FluentBase::<dyn Fluent>::init(self.source_id,
                                       name,
                                       fluent,
                                       fluent_rx,
                                       self.request_tx.clone(),
                                       self.sink_tx.clone()).run();
      }

      // handle incoming messages
      while let Some(broker_message) = self.source_rx.recv().await {
        let matcher_msg = broker_message.clone();
        match matcher_msg {
          BrokerMessage::Data(_) => {
            for (_, fluent_tx) in &self.fluents {
              fluent_tx.send(broker_message.clone()).await.unwrap();
            }
          }
          BrokerMessage::FluentReq(fluent_request) => {
            self.fluents[&fluent_request.name].send(broker_message)
                                              .await
                                              .unwrap();
          }
        }
      }
    });
  }
}
