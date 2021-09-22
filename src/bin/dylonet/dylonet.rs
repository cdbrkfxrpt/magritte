// Copyright 2021 Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use magritte::{DataPoint, StatusMessage};

use super::dylo_actor::{DyloActor, Update};
use eyre::Result;
use std::collections::{BTreeMap, HashMap};
use tracing::info;


#[derive(Default, Debug)]
pub struct Dylonet {
  status_message:  String,
  actors_by_mmsi:  HashMap<i32, DyloActor>,
  actors_by_value: BTreeMap<i64, BTreeMap<i64, Vec<i32>>>,
}

impl Dylonet {
  pub fn new() -> Self {
    info!("Dylonet spawning");
    Self { status_message:  String::from("dylonet alive"),
           actors_by_mmsi:  HashMap::new(),
           actors_by_value: BTreeMap::new(), }
  }

  pub fn receive(&mut self, dp: DataPoint) -> Result<()> {
    info!("received datapoint: {:?}", dp);

    match self.actors_by_mmsi.get_mut(&dp.mmsi) {
      Some(actor) => actor.send(),
      None => {
        self.actors_by_mmsi
            .insert(dp.mmsi, DyloActor::new(dp.mmsi, dp.ts, dp.lat, dp.lon));
      }
    };

    Ok(())
  }

  pub fn status_message(&mut self) -> StatusMessage {
    info!("responding: {:?}", self);
    StatusMessage { content: self.status_message.clone(), }
  }
}
