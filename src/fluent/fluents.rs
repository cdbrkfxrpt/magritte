// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Fluent, Memory, RequestSender};
use crate::types::{Message, RuleResult};

use std::collections::HashMap;
use tokio::{sync::mpsc, task::JoinHandle};
// use tracing::info;


pub fn build_fluents_index() -> HashMap<String, Box<dyn Fluent>> {
  let mut fluent_index: HashMap<String, Box<dyn Fluent>> = HashMap::new();

  fluent_index.insert("nearCoast".to_owned(), Box::new(NearCoast));
  fluent_index.insert("highSpeedNC".to_owned(), Box::new(HighSpeedNearCoast));
  // fluent_index.insert("neutral_fluent".to_owned(), Box::new(NeutralFluent));

  fluent_index
}


#[derive(Debug)]
pub struct NearCoast;

impl Fluent for NearCoast {
  fn rule(&self,
          message: &Message,
          _: &Memory,
          request_tx: RequestSender)
          -> JoinHandle<RuleResult> {
    let message = message.clone();
    tokio::spawn(async move {
      let (response_tx, mut response_rx) = mpsc::channel(32);

      let Message::Datapoint { source_id,
                               timestamp,
                               values } = message else {
        panic!("Message passed to rule is not a Datapoint");
      };

      let request = Message::new_knowledge_request("distance_from_coastline",
                                                   source_id,
                                                   timestamp,
                                                   vec![values["lon"],
                                                        values["lat"]],
                                                   response_tx);

      request_tx.send(request).await.unwrap();

      if let Some(response) = response_rx.recv().await {
        let Message::Response { values, .. } = response else {
          panic!("something something Response panic");
        };

        if values["distance"] <= 300.0 {
          return RuleResult::Boolean(true);
        }
      }

      RuleResult::Boolean(false)
    })
  }
}


#[derive(Debug)]
pub struct HighSpeedNearCoast;

impl Fluent for HighSpeedNearCoast {
  fn rule(&self,
          message: &Message,
          _: &Memory,
          request_tx: RequestSender)
          -> JoinHandle<RuleResult> {
    let message = message.clone();
    tokio::spawn(async move {
      let (response_tx, mut response_rx) = mpsc::channel(32);

      let Message::Datapoint { source_id, timestamp, values } = message else {
        panic!("Message passed to rule is not a Datapoint");
      };

      let request = Message::new_source_request("nearCoast",
                                                Some(source_id),
                                                timestamp,
                                                Vec::new(),
                                                response_tx);

      request_tx.send(request).await.unwrap();

      if let Some(response) = response_rx.recv().await {
        let Message::Response { rule_result, .. } = response else {
          panic!("something something Response panic");
        };

        let RuleResult::Boolean(near_coast) = rule_result else {
          panic!("something something RuleResult panic");
        };

        if near_coast && values["speed"] > 5.0 {
          return RuleResult::Boolean(true);
        }
      }

      RuleResult::Boolean(false)
    })
  }
}


#[derive(Debug)]
pub struct NeutralFluent;

impl Fluent for NeutralFluent {
  fn rule(&self,
          _: &Message,
          _: &Memory,
          _: RequestSender)
          -> JoinHandle<RuleResult> {
    tokio::spawn(async move { RuleResult::Boolean(true) })
  }
}


// TODO WRITE SOME TESTS
//
// also cool here! highlight that Rust enables easy test cases for fluents!
// it's a feature!
//
