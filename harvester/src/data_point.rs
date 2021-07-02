// Copyright 2021 bmc::labs GmbH.
//
// Authors:
//   Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use getset::Getters;
use std::fmt;
use tokio_postgres::row::Row;


#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct DataPoint {
  id:      i64,
  mmsi:    i32,
  status:  i32,
  turn:    f64,
  speed:   f64,
  course:  f64,
  heading: i32,
  lon:     f64,
  lat:     f64,
  ts:      i64,
}

impl DataPoint {
  pub fn from_row(row: Row) -> Self {
    Self { id:      row.get("id"),
           mmsi:    row.get("mmsi"),
           status:  row.get("status"),
           turn:    row.get("turn"),
           speed:   row.get("speed"),
           course:  row.get("course"),
           heading: row.get("heading"),
           lon:     row.get("lon"),
           lat:     row.get("lat"),
           ts:      row.get("ts"), }
  }
}

impl fmt::Display for DataPoint {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f,
           "DataPoint {{ id: {:8}, mmsi: {}, status: {:2}, turn: {:+7.1}, \
            speed: {:4.1}, course: {:5.1}, heading: {:3}, lon: {:+9.5}, lat: \
            {:+9.5}, ts: {} }}",
           self.id,
           self.mmsi,
           self.status,
           self.turn,
           self.speed,
           self.course,
           self.heading,
           self.lon,
           self.lat,
           self.ts)
  }
}
