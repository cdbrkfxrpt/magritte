// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Fluent, Memory, RequestSender};
use crate::types::{Message, RuleResult};

// use std::collections::HashMap;
// use tokio::sync::mpsc;
use tokio::task::JoinHandle;


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
//           -> Option<RuleResult> {
//     let (tx, mut rx) = mpsc::channel(32);

//     let request = RequestBody::new("neutral_fluent",
//                                    None,
//                                    message.timestamp,
//                                    HashMap::new(),
//                                    tx.clone());

//     self.send(message.to_source_request(request), request_tx);

//     Some(RuleResult::Boolean(true))
//   }
// }


// #[derive(Debug)]
// pub struct NearCoast {}

// impl Fluent for NearCoast {
//   fn rule(&self, datapoint: Datapoint)
// }


// TODO WRITE SOME TESTS
//
// also cool here! highlight that Rust enables easy test cases for fluents!
// it's a feature!
//
