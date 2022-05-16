// Copyright 2022 Florian Eich <florian.eich@gmail.com>
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use std::collections::HashMap;
use tokio_postgres::row::Row;


#[derive(Clone, Debug)]
pub struct Datapoint {
  pub source_id: usize,
  pub timestamp: usize,
  pub values:    HashMap<String, f64>,
}


impl Datapoint {
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
    self.source_id = row.get::<&str, i32>("source_id") as usize;
    self.timestamp = row.get::<&str, i64>("timestamp") as usize;
    for (value_name, value) in self.values.iter_mut() {
      *value = row.get(value_name.as_str());
    }
  }
}
