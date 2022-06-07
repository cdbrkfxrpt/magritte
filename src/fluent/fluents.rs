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
  fluent_index.insert("nearPorts".to_owned(), Box::new(NearPorts));
  fluent_index.insert("highSpeedNC".to_owned(), Box::new(HighSpeedNearCoast));
  fluent_index.insert("rendezVousConditions".to_owned(),
                      Box::new(RendezVousConditions));
  fluent_index.insert("rendezVous".to_owned(), Box::new(RendezVous));
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
          -> JoinHandle<(bool, Option<Vec<f64>>)> {
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
          return (true, None);
        }
      }

      (false, None)
    })
  }
}


#[derive(Debug)]
pub struct NearPorts;

impl Fluent for NearPorts {
  fn rule(&self,
          datapoint: Datapoint,
          _: &Memory,
          request_tx: RequestSender)
          -> JoinHandle<(bool, Option<Vec<f64>>)> {
    tokio::spawn(async move {
      let (response_tx, mut response_rx) = mpsc::channel(32);

      let request =
        KnowledgeRequest::new("distance_from_ports",
                              datapoint.source_id,
                              datapoint.timestamp,
                              vec![Box::new(datapoint.values["lon"]),
                                   Box::new(datapoint.values["lat"])],
                              response_tx);

      request_tx.send(BrokerRequest::KnowledgeReq(request))
                .await
                .unwrap();

      while let Some(response) = response_rx.recv().await {
        if response.values.get::<&str, f64>("distance") <= 300.0 {
          return (true, None);
        }
      }

      (false, None)
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
          -> JoinHandle<(bool, Option<Vec<f64>>)> {
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
          return (true, None);
        }
      }

      (false, None)
    })
  }
}


#[derive(Debug)]
pub struct RendezVousConditions;

impl Fluent for RendezVousConditions {
  fn rule(&self,
          datapoint: Datapoint,
          _: &Memory,
          request_tx: RequestSender)
          -> JoinHandle<(bool, Option<Vec<f64>>)> {
    tokio::spawn(async move {
      // first, check if we're at low speed/stopped - if not, return early
      if datapoint.values["speed"] > 0.5 {
        return (false, None);
      }

      // second we check if we're a Pilot or Tug vessel
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
        // Vessel Type Codes:
        // - 30: Fishing
        // - 31: Tug
        // - 32: Tug
        // - 35: Military
        // - 50: Pilot
        // - 51: SAR Vessel
        // - 52: Tug
        // - 53: Port Tender
        // - 55: Law Enforcement

        // the following code implements the rendezVous condition:
        // !isTug && !isPilot
        let type_codes = vec![31, 32, 50, 52];
        if type_codes.contains(&response.values.get("ship_type")) {
          info!("is Tug or Pilot");
          return (false, None);
        }
      }

      // then check if we are near coast or near ports
      let (response_tx, mut response_rx) = mpsc::channel(32);
      let request = FluentRequest::new(Some(datapoint.source_id),
                                       datapoint.timestamp,
                                       "nearPorts",
                                       None,
                                       response_tx.clone());

      request_tx.send(BrokerRequest::FluentReq(request))
                .await
                .unwrap();

      let near_ports = match response_rx.recv().await {
        Some(near_ports) => near_ports.holds,
        None => false,
      };

      let (response_tx, mut response_rx) = mpsc::channel(32);
      let request = FluentRequest::new(Some(datapoint.source_id),
                                       datapoint.timestamp,
                                       "nearCoast",
                                       None,
                                       response_tx.clone());

      request_tx.send(BrokerRequest::FluentReq(request))
                .await
                .unwrap();

      let near_coast = match response_rx.recv().await {
        Some(near_coast) => near_coast.holds,
        None => false,
      };

      // the following code implements the rendezVous condition:
      // !nearPorts && !nearCoast
      if near_ports || near_coast {
        return (false, None);
      }

      (true, Some(vec![datapoint.values["lon"], datapoint.values["lat"]]))
    })
  }
}


#[derive(Debug)]
pub struct RendezVous;

impl Fluent for RendezVous {
  fn rule(&self,
          datapoint: Datapoint,
          _: &Memory,
          request_tx: RequestSender)
          -> JoinHandle<(bool, Option<Vec<f64>>)> {
    tokio::spawn(async move {
      // distance calculation here is from Rust Cookbook:
      // https://rust-lang-nursery.github.io/rust-cookbook/science/mathematics/trigonometry.html#distance-between-two-points-on-the-earth
      let radius_earth_km = 6371.0_f64;

      let (this_lon, this_lat) =
        (datapoint.values["lon"], datapoint.values["lat"]);
      let this_lat_rad = this_lat.to_radians();

      let (response_tx, mut response_rx) = mpsc::channel(32);
      let request = FluentRequest::new(None,
                                       datapoint.timestamp,
                                       "rendezVousConditions",
                                       None,
                                       response_tx.clone());
      request_tx.send(BrokerRequest::FluentReq(request))
                .await
                .unwrap();

      while let Some(fluent_result) = response_rx.recv().await {
        info!("got response from {:?}", fluent_result.source_id);
        if fluent_result.holds {
          let other_params = fluent_result.params.unwrap();
          let (other_lon, other_lat) = (other_params[0], other_params[1]);
          let other_lat_rad = other_lat.to_radians();

          let d_lon = (this_lon - other_lon).to_radians();
          let d_lat = (this_lat - other_lat).to_radians();

          let central_angle_inner = (d_lat / 2.0).sin().powi(2)
                                    + this_lat_rad.cos()
                                      * other_lat_rad.cos()
                                      * (d_lon / 2.0).sin().powi(2);

          let central_angle = 2.0 * central_angle_inner.sqrt().asin();

          let distance = radius_earth_km * central_angle;

          if distance < 50.0 {
            return (true, None);
          }
        }
      }

      (false, None)
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
          -> JoinHandle<(bool, Option<Vec<f64>>)> {
    tokio::spawn(async move { (true, None) })
  }
}


// TODO WRITE SOME TESTS
//
// also cool here! highlight that Rust enables easy test cases for fluents!
// it's a feature!
//
