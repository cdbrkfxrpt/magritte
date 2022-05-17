// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

mod fluents;
pub use fluents::NeutralFluent;

use crate::types::{Datapoint, FluentResult, Message};

use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::info;


type FluentIndex = HashMap<String, Box<dyn Fluent>>;

pub trait Fluent: Send + core::fmt::Debug {
  fn rule(&self, datapoint: Datapoint) -> Option<FluentResult>;
}


pub fn build_index() -> FluentIndex {
  let mut fluent_index: FluentIndex = HashMap::new();

  fluent_index.insert("neutral_fluent".to_owned(), Box::new(NeutralFluent));
  // fluent_index.insert("near_coast".to_owned(), Box::new(NearCoast));

  fluent_index
}


#[derive(Debug)]
pub struct FluentBase<T: 'static + Send + ?Sized + Fluent> {
  source_id:  usize,
  name:       String,
  fluent:     Box<T>,
  fluent_rx:  mpsc::Receiver<Message>,
  message_tx: mpsc::Sender<Message>,
  sink_tx:    mpsc::Sender<FluentResult>,
}

impl<T: 'static + Send + ?Sized + Fluent> FluentBase<T> {
  pub fn init(source_id: usize,
              name: String,
              fluent: Box<T>,
              fluent_rx: mpsc::Receiver<Message>,
              message_tx: mpsc::Sender<Message>,
              sink_tx: mpsc::Sender<FluentResult>)
              -> Self {
    Self { source_id,
           name,
           fluent,
           fluent_rx,
           message_tx,
           sink_tx }
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      while let Some(message) = self.fluent_rx.recv().await {
        match message {
          Message::Data(datapoint) => {
            let rule_eval = self.fluent.rule(datapoint);
            match rule_eval {
              Some(fluent_result) => {
                self.sink_tx.send(fluent_result).await.unwrap();
              }
              None => {
                info!("fluent {} produced None result on source {}",
                      self.name, self.source_id)
              }
            }
          }
          _ => (),
        }
      }
    });
  }
}
