// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{fluent::{build_index, Fluent, FluentBase},
            types::{FluentResult, Message}};

use std::collections::HashMap;
use tokio::sync::mpsc;


#[derive(Debug)]
pub struct Source {
  source_id:  usize,
  source_rx:  mpsc::Receiver<Message>,
  message_tx: mpsc::Sender<Message>,
  sink_tx:    mpsc::Sender<FluentResult>,
  fluents:    HashMap<String, mpsc::Sender<Message>>,
}

impl Source {
  pub fn init(source_id: usize,
              source_rx: mpsc::Receiver<Message>,
              message_tx: mpsc::Sender<Message>,
              sink_tx: mpsc::Sender<FluentResult>)
              -> Self {
    let fluents = HashMap::new();

    Self { source_id,
           source_rx,
           message_tx,
           sink_tx,
           fluents }
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      for (name, fluent) in build_index() {
        let (fluent_tx, fluent_rx) = mpsc::channel(32);
        self.fluents.insert(name.clone(), fluent_tx);

        FluentBase::<dyn Fluent>::init(self.source_id,
                                       name,
                                       fluent,
                                       fluent_rx,
                                       self.message_tx.clone(),
                                       self.sink_tx.clone()).run();
      }

      while let Some(message) = self.source_rx.recv().await {
        for (_, fluent_tx) in self.fluents.clone() {
          fluent_tx.send(message.clone()).await.unwrap();
        }
      }
    });
  }
}
