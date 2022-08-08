// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use serde::Deserialize;


#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Holds parameters for the `Source` service of the application, which reads
/// data from the source (i.e. the PostgreSQL database) and publishes it to the
/// `Broker` service.
pub struct Source {
  run_params:   RunParams,
  query_params: QueryParams,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Holds parameters for the execution of the `Source` service.
struct RunParams {
  pub millis_per_cycle:  u64,
  pub datapoints_to_run: usize,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Holds parameters for the database query performed by the `Source` service.
struct QueryParams {
  pub key_name:       String,
  pub timestamp_name: String,
  pub fluent_names:   Vec<String>,
  pub from_table:     String,
  pub order_by:       String,
}


// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::{QueryParams, RunParams, Source};
  use crate::util::stringvec;

  use pretty_assertions::assert_eq;

  #[test]
  fn source_params_test() {
    let millis_per_cycle = 42;
    let datapoints_to_run = 1337;

    let rp = RunParams { millis_per_cycle,
                         datapoints_to_run };

    assert_eq!(rp.millis_per_cycle, millis_per_cycle);
    assert_eq!(rp.datapoints_to_run, datapoints_to_run);

    let key_name = String::from("id");
    let timestamp_name = String::from("ts");
    let fluent_names = stringvec!["lat", "lon", "speed"];
    let from_table = String::from("the.matrix");
    let order_by = String::from("serial");

    let qp = QueryParams { key_name:       key_name.clone(),
                           timestamp_name: timestamp_name.clone(),
                           fluent_names:   fluent_names.clone(),
                           from_table:     from_table.clone(),
                           order_by:       order_by.clone(), };

    assert_eq!(qp.key_name, key_name);
    assert_eq!(qp.timestamp_name, timestamp_name);
    assert_eq!(qp.fluent_names, fluent_names);
    assert_eq!(qp.from_table, from_table);
    assert_eq!(qp.order_by, order_by);

    let src = Source { run_params:   rp.clone(),
                       query_params: qp.clone(), };

    assert_eq!(src.run_params, rp);
    assert_eq!(src.query_params, qp);
  }
}
