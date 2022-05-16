// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

// use crate::types::{Datapoint, Message, Request};
use crate::types::Message;

use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::info;


#[derive(Debug)]
pub struct Source {
  source_rx:    mpsc::Receiver<Message>,
  message_tx:   mpsc::Sender<Message>,
  fluent_index: HashMap<String, mpsc::Sender<Message>>,
}

impl Source {
  pub fn init(source_rx: mpsc::Receiver<Message>,
              message_tx: mpsc::Sender<Message>)
              -> Self {
    let fluent_index = HashMap::new();
    Self { source_rx,
           message_tx,
           fluent_index }
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      while let Some(message) = self.source_rx.recv().await {
        info!(?message);
      }
    });
  }
}
