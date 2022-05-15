// Copyright 2022 Florian Eich <florian.eich@gmail.com>
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use getset::Getters;
use std::collections::HashMap;
use tokio_postgres::row::Row;


#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct DataPoint {
  source_id: i32,
  timestamp: i64,
  values:    HashMap<String, f64>,
}


impl DataPoint {
  pub fn new(value_names: &Vec<String>) -> Self {
    let (source_id, timestamp) = (0, 0);
    let mut values = HashMap::new();
    for value_name in value_names {
      values.insert(value_name.to_owned(), 0.0f64);
    }

    Self { source_id,
           timestamp,
           values }
  }

  pub fn update(&mut self, row: Row) {
    self.source_id = row.get("source_id");
    self.timestamp = row.get("timestamp");
    for (value_name, value) in self.values.iter_mut() {
      *value = row.get(value_name.as_str());
    }
  }
}
