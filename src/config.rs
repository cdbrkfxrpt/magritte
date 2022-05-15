// Cloneright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct Config {
  pub feeder: FeederConfig,
}


#[derive(Clone, Debug, Deserialize)]
pub struct FeederConfig {
  pub channel_capacity: usize,
  pub millis_per_cycle: u64,
  pub connection:       FeederConnection,
  pub query:            FeederQuery,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FeederConnection {
  pub host:     String,
  pub user:     String,
  pub password: String,
  pub dbname:   String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FeederQuery {
  pub source_id:   String,
  pub timestamp:   String,
  pub value_names: Vec<String>,
  pub from_table:  String,
  pub order_by:    String,
}
