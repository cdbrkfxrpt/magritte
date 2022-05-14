// Copyright 2022 Florian Eich <florian.eich@gmail.com>
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
  id:        i64,
  source:    i32,
  timestamp: i64,
  lon:       f64,
  lat:       f64,
  speed:     f64,
}


impl DataPoint {
  pub fn from_row(row: Row) -> Self {
    Self { id:        row.get("id"),
           source:    row.get("source"),
           timestamp: row.get("timestamp"),
           lon:       row.get("lon"),
           lat:       row.get("lat"),
           speed:     row.get("speed"), }
  }
}

impl fmt::Display for DataPoint {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f,
           "DataPoint {{\n  source: {}, timestamp: {}, lon: {:+7.3}, lat: \
            {:+7.3}, speed: {:4.1}\n}}",
           self.source, self.timestamp, self.lon, self.lat, self.speed)
  }
}
