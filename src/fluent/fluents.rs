// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Fluent, Memory, RequestSender};
use crate::types::{BrokerRequest,
                   Datapoint,
                   FluentRequest,
                   KnowledgeRequest};

use std::collections::HashMap;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::info;


pub fn build_fluents_index() -> HashMap<String, Box<dyn Fluent>> {
  let mut fluent_index: HashMap<String, Box<dyn Fluent>> = HashMap::new();

  fluent_index.insert("nearCoast".to_owned(), Box::new(NearCoast));
  fluent_index.insert("highSpeedNC".to_owned(), Box::new(HighSpeedNearCoast));
  fluent_index.insert("rendezvous".to_owned(), Box::new(Rendezvous));
  // fluent_index.insert("neutral_fluent".to_owned(), Box::new(NeutralFluent));

  fluent_index
}


#[derive(Debug)]
pub struct NearCoast;

impl Fluent for NearCoast {
  fn rule(&self,
          datapoint: Datapoint,
          _: &Memory,
          request_tx: RequestSender)
          -> JoinHandle<bool> {
    tokio::spawn(async move {
      let (response_tx, mut response_rx) = mpsc::channel(32);

      let request =
        KnowledgeRequest::new("distance_from_coastline",
                              datapoint.source_id,
                              datapoint.timestamp,
                              vec![Box::new(datapoint.values["lon"]),
                                   Box::new(datapoint.values["lat"])],
                              response_tx);

      request_tx.send(BrokerRequest::KnowledgeReq(request))
                .await
                .unwrap();

      if let Some(response) = response_rx.recv().await {
        if response.values.get::<&str, f64>("distance") <= 300.0 {
          return true;
        }
      }

      false
    })
  }
}


#[derive(Debug)]
pub struct HighSpeedNearCoast;

impl Fluent for HighSpeedNearCoast {
  fn rule(&self,
          datapoint: Datapoint,
          _: &Memory,
          request_tx: RequestSender)
          -> JoinHandle<bool> {
    tokio::spawn(async move {
      let (response_tx, mut response_rx) = mpsc::channel(32);

      let request = FluentRequest::new(Some(datapoint.source_id),
                                       datapoint.timestamp,
                                       "nearCoast",
                                       None,
                                       response_tx);

      request_tx.send(BrokerRequest::FluentReq(request))
                .await
                .unwrap();

      if let Some(near_coast) = response_rx.recv().await {
        if near_coast.holds && datapoint.values["speed"] > 5.0 {
          return true;
        }
      }

      false
    })
  }
}


#[derive(Debug)]
pub struct Rendezvous;

impl Fluent for Rendezvous {
  fn rule(&self,
          datapoint: Datapoint,
          _: &Memory,
          request_tx: RequestSender)
          -> JoinHandle<bool> {
    tokio::spawn(async move {
      let (response_tx, mut response_rx) = mpsc::channel(32);

      let request = KnowledgeRequest::new("ship_type",
                                          datapoint.source_id,
                                          datapoint.timestamp,
                                          vec![Box::new(datapoint.source_id
                                                        as i32)],
                                          response_tx);

      request_tx.send(BrokerRequest::KnowledgeReq(request))
                .await
                .unwrap();

      if let Some(response) = response_rx.recv().await {
        info!(?response.values);

        if vec![31, 32, 52, 50].contains(&response.values.get("ship_type")) {
          info!("is Tug or Pilot");
          return false;
        }
      }

      false
    })
  }
}


#[derive(Debug)]
pub struct NeutralFluent;

impl Fluent for NeutralFluent {
  fn rule(&self,
          _: Datapoint,
          _: &Memory,
          _: RequestSender)
          -> JoinHandle<bool> {
    tokio::spawn(async move { true })
  }
}


// TODO WRITE SOME TESTS
//
// also cool here! highlight that Rust enables easy test cases for fluents!
// it's a feature!
//
