// Copyright 2021 Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use actix::prelude::*;
use eyre::Result;


#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct Update {
  ts:  i64,
  lat: f64,
  lon: f64,
}


#[derive(Debug)]
pub struct DyloActor {
  mmsi: i32,
  ts:   i64,
  lat:  f64,
  lon:  f64,
}

impl DyloActor {
  pub fn new(mmsi: i32, ts: i64, lat: f64, lon: f64) -> Self {
    Self { mmsi, ts, lat, lon }
  }
}

impl Actor for DyloActor {
  type Context = Context<Self>;
}

impl Handler<Update> for DyloActor {
  type Result = Result<()>;

  fn handle(&mut self,
            update: Update,
            _ctx: &mut Context<Self>)
            -> Self::Result {
    self.ts = update.ts;
    self.lat = update.lat;
    self.lon = update.lon;
    Ok(())
  }
}
