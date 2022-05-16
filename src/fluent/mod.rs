// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

mod neutral_fluent;
pub use neutral_fluent::NeutralFluent;

use crate::types::{Datapoint, FluentResult, Message};

use core::fmt;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::info;


type FluentIndex = HashMap<String, Box<dyn Fluent>>;


pub fn build_index() -> FluentIndex {
  let mut fluent_index: FluentIndex = HashMap::new();

  fluent_index.insert("neutral_fluent".to_owned(), Box::new(NeutralFluent));

  fluent_index
}


#[derive(Debug)]
pub struct FluentBase<T: 'static + Send + ?Sized + Fluent> {
  fluent_rx:  mpsc::Receiver<Message>,
  message_tx: mpsc::Sender<Message>,
  // sink_tx:    mpsc::Sender<FluentResult>,
  fluent:     Box<T>,
}

impl<T: 'static + Send + ?Sized + Fluent> FluentBase<T> {
  pub fn init(fluent_rx: mpsc::Receiver<Message>,
              message_tx: mpsc::Sender<Message>,
              // sink_tx: mpsc::Sender<FluentResult>,
              fluent: Box<T>)
              -> Self {
    Self { fluent_rx,
           message_tx,
           // sink_tx,
           fluent }
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      while let Some(message) = self.fluent_rx.recv().await {
        if let Message::Data(datapoint) = message {
          let fluent_result = self.fluent.rule(datapoint);
          info!(?fluent_result);
        }
      }
    });
  }
}


pub trait Fluent: Send {
  fn rule(&self, datapoint: Datapoint) -> FluentResult;
}

impl fmt::Debug for dyn Fluent {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "I'm a Fluent")
  }
}
