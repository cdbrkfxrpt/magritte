// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

mod fluents;
pub use fluents::build_fluents_index;

use crate::types::{BrokerMessage, BrokerRequest, Datapoint, FluentResult};

use circular_queue::CircularQueue;
use std::marker::Sync;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::info;


type Memory = CircularQueue<FluentResult>;
type RequestSender = mpsc::Sender<BrokerRequest>;

pub trait Fluent: Send + core::fmt::Debug + Sync {
  fn rule(&self,
          datapoint: Datapoint,
          memory: &Memory,
          request_tx: RequestSender)
          -> JoinHandle<(bool, Option<Vec<f64>>)>;
}


#[derive(Debug)]
pub struct FluentBase<T: 'static + Send + ?Sized + Sync + Fluent> {
  source_id:  usize,
  name:       String,
  fluent:     Box<T>,
  fluent_rx:  mpsc::Receiver<BrokerMessage>,
  request_tx: RequestSender,
  sink_tx:    mpsc::Sender<FluentResult>,
  memory:     Memory,
}

impl<T: 'static + Send + ?Sized + Sync + Fluent> FluentBase<T> {
  pub fn init(source_id: usize,
              name: String,
              fluent: Box<T>,
              fluent_rx: mpsc::Receiver<BrokerMessage>,
              request_tx: RequestSender,
              sink_tx: mpsc::Sender<FluentResult>)
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
          BrokerMessage::Data(datapoint) => {
            let rule_result =
              self.fluent
                  .rule(datapoint.clone(),
                        &self.memory,
                        self.request_tx.clone())
                  .await;

            match rule_result {
              Ok((holds, params)) => {
                let fluent_result = FluentResult::new(datapoint.source_id,
                                                      datapoint.timestamp,
                                                      &self.name,
                                                      holds,
                                                      params);
                self.memory.push(fluent_result.clone());

                let only_holding_to_sink = false; // TODO take from config
                if holds || !only_holding_to_sink {
                  self.sink_tx.send(fluent_result).await.unwrap();
                }
              }
              Err(_) => {
                info!("no result on {}-{}", self.source_id, self.name);
              }
            }
          }
          BrokerMessage::FluentReq(fluent_request) => {
            // NOTE START
            //
            // check if a source_id was passed...
            let fluent_result =
              if let Some(source_id) = fluent_request.source_id {
                // ... then check if it's the same as "ours"...
                if source_id == self.source_id {
                  // ... if so:
                  // search for the matching response and take it, or None
                  self.memory
                      .iter()
                      .find(|&fluent_result| {
                        fluent_request.timestamp == fluent_result.timestamp
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
            if let Some(fluent_result) = fluent_result {
              fluent_request.response_tx
                            .send(fluent_result.clone())
                            .await
                            .unwrap();
            }
            // NOTE END
          }
        }
      }
    });
  }
}
