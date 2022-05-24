// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

mod fluents;
pub use fluents::build_fluents_index;

use crate::types::{Message, RuleResult};

use circular_queue::CircularQueue;
use std::marker::Sync;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::info;


type Memory = CircularQueue<Message>;
type RequestSender = mpsc::Sender<Message>;

pub trait Fluent: Send + core::fmt::Debug + Sync {
  fn rule(&self,
          message: &Message,
          memory: &Memory,
          request_tx: RequestSender)
          -> JoinHandle<RuleResult>;
}


#[derive(Debug)]
pub struct FluentBase<T: 'static + Send + ?Sized + Sync + Fluent> {
  source_id:  usize,
  name:       String,
  fluent:     Box<T>,
  fluent_rx:  mpsc::Receiver<Message>,
  request_tx: mpsc::Sender<Message>,
  sink_tx:    mpsc::Sender<Message>,
  memory:     Memory,
}

impl<T: 'static + Send + ?Sized + Sync + Fluent> FluentBase<T> {
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
                  .rule(&message, &self.memory, self.request_tx.clone())
                  .await;

            match rule_result {
              Ok(rule_result) => {
                let message = message.datapoint_to_response(self.name.clone(),
                                                            rule_result);
                info!(?message);

                self.memory.push(message.clone());
                self.sink_tx.send(message).await.unwrap();
              }
              Err(_) => {
                info!("no result on {}-{}", self.source_id, self.name);
              }
            }
          }
          Message::Request { source_id,
                             timestamp,
                             response_tx,
                             .. } => {
            // NOTE START
            //
            // check if a source_id was passed...
            let response = if let Some(source_id) = source_id {
              // ... then check if it's the same as "ours"...
              if source_id == self.source_id {
                // ... if so:
                // search for the matching response and take it, or None
                self.memory.iter().find(|&r| {
                  let Message::Response { timestamp: ts, .. } = r else {
                    panic!("Message from memory is not a Response");
                  };
                  ts == &timestamp
                })
              } else {
                // ... if not:
                // take the latest response we have, or None
                self.memory.iter().next()
              }
            } else {
              // ... and if no source_id was passed at all:
              // take the latest response we have, or None
              self.memory.iter().next()
            };

            // if we found a response we can return, send it.
            // otherwise everything gets dropped here, the request channel is
            // closed, and if all fluents do this the requesting entity learns
            // via all channels being closed that no response is available.
            if let Some(response) = response {
              response_tx.send(response.clone()).await.unwrap();
            }
            // NOTE END
          }
          _ => (),
        }
      }
    });
  }
}
