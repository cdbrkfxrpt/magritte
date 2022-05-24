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

  // fluent_index.insert("neutral_fluent".to_owned(), Box::new(NeutralFluent));
  // fluent_index.insert("none_fluent".to_owned(), Box::new(NoneFluent));
  // fluent_index.insert("request_neutral".to_owned(),
  // Box::new(RequestNeutral));
  fluent_index.insert("near_coast".to_owned(), Box::new(NearCoast));

  fluent_index
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


// #[derive(Debug)]
// pub struct NoneFluent;

// impl Fluent for NoneFluent {
//   fn rule(&self,
//           _: &Message,
//           _: &Memory,
//           _: RequestSender)
//           -> Option<RuleResult> {
//     None
//   }
// }


// #[derive(Debug)]
// pub struct RequestNeutral;

// impl Fluent for RequestNeutral {
//   fn rule(&self,
//           message: &Message,
//           _: &Memory,
//           request_tx: RequestSender)
//           -> JoinHandle<RuleResult> {
//     let message = message.clone();
//     tokio::spawn(async move {
//       let (response_tx, mut response_rx) = mpsc::channel(32);

//       let Message::Datapoint { timestamp, .. } = message else {
//         panic!("Message passed to rule is not a Datapoint");
//       };

//       let request = Message::new_source_request("neutral_fluent",
//                                                 None,
//                                                 timestamp.clone(),
//                                                 Vec::new(),
//                                                 response_tx);
//       request_tx.send(request).await.unwrap();

//       while let Some(response) = response_rx.recv().await {
//         info!(?response);
//       }

//       RuleResult::Boolean(true)
//     })
//   }
// }


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

      let Message::Datapoint { timestamp, values, .. } = message else {
        panic!("Message passed to rule is not a Datapoint");
      };

      let request = Message::new_knowledge_request("distance_from_coastline",
                                                   None,
                                                   timestamp,
                                                   vec![values["lon"],
                                                        values["lat"]],
                                                   response_tx);

      request_tx.send(request).await.unwrap();

      if let Some(response) = response_rx.recv().await {
        let Message::Response { values, .. } = response else {
          panic!("something something response panic");
        };

        if values["distance"] <= 300.0 {
          return RuleResult::Boolean(true);
        }
      }

      RuleResult::Boolean(false)
    })
  }
}


// TODO WRITE SOME TESTS
//
// also cool here! highlight that Rust enables easy test cases for fluents!
// it's a feature!
//
