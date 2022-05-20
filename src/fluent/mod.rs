// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

mod fluents;
pub use fluents::{NeutralFluent, NoneFluent};

use crate::types::{Message, RuleResult};

use circular_queue::CircularQueue;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::info;


type FluentIndex = HashMap<String, Box<dyn Fluent>>;
type Memory = CircularQueue<Message>;
type RequestSender = mpsc::Sender<Message>;

pub trait Fluent: Send + core::fmt::Debug {
  fn rule(&self,
          message: &Message,
          memory: &Memory,
          request_tx: RequestSender)
          -> Option<RuleResult>;

  // fn send(&self, message: Message, request_tx: RequestSender) {
  //   tokio::spawn(async move {
  //     request_tx.send(message).await.unwrap();
  //   });
  // }

  // fn request(&self, fluent_name: &str, from_source: Option<usize>, )
}


pub fn build_index() -> FluentIndex {
  let mut fluent_index: FluentIndex = HashMap::new();

  fluent_index.insert("neutral_fluent".to_owned(), Box::new(NeutralFluent));
  // fluent_index.insert("none_fluent".to_owned(), Box::new(NoneFluent));

  fluent_index
}


#[derive(Debug)]
pub struct FluentBase<T: 'static + Send + ?Sized + Fluent> {
  source_id:  usize,
  name:       String,
  fluent:     Box<T>,
  fluent_rx:  mpsc::Receiver<Message>,
  request_tx: mpsc::Sender<Message>,
  sink_tx:    mpsc::Sender<Message>,
  memory:     Memory,
}

impl<T: 'static + Send + ?Sized + Fluent> FluentBase<T> {
  pub fn init(source_id: usize,
              name: String,
              fluent: Box<T>,
              fluent_rx: mpsc::Receiver<Message>,
              request_tx: mpsc::Sender<Message>,
              sink_tx: mpsc::Sender<Message>)
              -> Self {
    Self { source_id,
           name,
           fluent,
           fluent_rx,
           request_tx,
           sink_tx,
           // TODO capacity from config
           memory: CircularQueue::with_capacity(32) }
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      while let Some(message) = self.fluent_rx.recv().await {
        match message {
          Message::Datapoint { .. } => {
            let rule_result =
              self.fluent
                  .rule(&message, &self.memory, self.request_tx.clone());

            match rule_result {
              Some(rule_result) => {
                let message =
                  message.to_response(self.name.clone(), rule_result);

                self.memory.push(message.clone());
                self.sink_tx.send(message).await.unwrap();
              }
              None => {
                info!("no result on {}-{}", self.source_id, self.name);
              }
            }
          }
          _ => (),
        }
      }
    });
  }
}
