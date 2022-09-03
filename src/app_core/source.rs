// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Node, NodeRx, NodeTx};
use crate::fluent::Fluent;

use eyre::{bail, eyre, Result};
use serde::Deserialize;
use std::time::Instant;
use tokio::time;
use tokio_postgres::Client;
use tracing::info;


const BIG_BANG: usize = 1_443_650_400;
const ARMAGEDDON: usize = 1_459_461_599;


#[derive(Debug, Deserialize)]
/// Reads data from the source (i.e. the PostgreSQL database) and publishes it
/// to the [`Broker`](super::broker::Broker) service.
pub struct Source {
  publishes:    Vec<String>,
  run_params:   RunParams,
  query_params: QueryParams,
  #[serde(skip)]
  node_tx:      Option<NodeTx>,
}

impl Source {
  /// Runs the [`Source`], retrieving data from the database and publishing
  /// fluents to the [`Broker`](super::broker::Broker). Consumes the original
  /// object.
  ///
  /// Data retrieval is performed using the following SQL:
  ///
  /// ```sql
  #[doc = include_str!("./sql/source.sql")]
  /// ```
  pub async fn run(self, database_client: Option<Client>) -> Result<()> {
    let database_client =
      database_client.ok_or(eyre!("Source requires a database client"))?;

    let node_tx = match self.node_tx {
      Some(node_tx) => node_tx,
      None => bail!("Source not initialized, aborting"),
    };

    let query_statement_raw =
      format!(include_str!("./sql/source.sql"),
              key_name = self.query_params.key_name,
              timestamp_name = self.query_params.timestamp_name,
              fluent_names = self.publishes.join(", "),
              from_table = self.query_params.from_table,
              order_by = self.query_params.order_by);

    let query_statement =
      match database_client.prepare(&query_statement_raw).await {
        Ok(statement) => statement,
        Err(err) => {
          // drop(self.fluent_tx);
          panic!("Error in Source PostgreSQL query: {}", err);
        }
      };

    info!("SQL statement prepared");

    let mut interval =
      time::interval(time::Duration::from_millis(self.run_params
                                                     .millis_per_cycle));

    let mut time: usize = BIG_BANG + self.run_params.starting_offset;
    let mut key: usize;

    while let Ok(rows) = database_client.query(&query_statement,
                                               &[&(time as i64)])
                                        .await
    {
      if !rows.is_empty() {
        // print number of rows --> "simultaneous data points"
        println!("data_points,{}", rows.len());
      }
      for row in rows {
        key = row.get::<&str, i32>("key") as usize;

        node_tx.send(Fluent::new("instant",
                                 &[key],
                                 time,
                                 Box::new(Instant::now())))?;

        for fluent_name in self.publishes.iter() {
          let value: f64 = row.get(fluent_name.as_str());
          let fluent =
            Fluent::new(&fluent_name, &[key], time, Box::new(value));

          node_tx.send(fluent)?;
        }
      }
      time += 1;

      if self.run_params.hours_to_run > 0
         && (time == BIG_BANG + self.run_params.hours_to_run * 3_600
             || time == ARMAGEDDON)
      {
        info!("ran requested timeframe ({} hours) or reached end of data",
              self.run_params.hours_to_run);
        return Ok(());
      }

      interval.tick().await;
    }

    Ok(())
  }
}

impl Node for Source {
  /// `Source` publishes fluents specified by name in the app configuration.
  fn publishes(&self) -> Vec<String> {
    let mut publishes = self.publishes.clone();
    publishes.push("instant".to_owned());
    publishes
  }

  /// `Source` subscribes to no fluents. Implmenetation returns empty `Vec`.
  fn subscribes_to(&self) -> Vec<String> {
    Vec::new()
  }

  /// `Source` requires only a sender handle since it subscribes to no fluents.
  fn initialize(&mut self, node_tx: NodeTx, _: NodeRx) {
    self.node_tx = Some(node_tx);
  }
}


#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Holds parameters for the execution of the [`Source`] service.
struct RunParams {
  pub starting_offset:  usize,
  pub millis_per_cycle: u64,
  pub hours_to_run:     usize,
}


#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Holds parameters for the database query performed by the [`Source`].
struct QueryParams {
  pub key_name:       String,
  pub timestamp_name: String,
  pub from_table:     String,
  pub order_by:       String,
}


// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::{Node, QueryParams, RunParams, Source};
  use crate::stringvec;

  use pretty_assertions::assert_eq;


  fn run_params() -> (usize, u64, usize) {
    let starting_offset = 1337;
    let millis_per_cycle = 42;
    let hours_to_run = 23;

    (starting_offset, millis_per_cycle, hours_to_run)
  }

  fn run_params_init() -> RunParams {
    let (starting_offset, millis_per_cycle, hours_to_run) = run_params();

    RunParams { starting_offset,
                millis_per_cycle,
                hours_to_run }
  }

  fn query_params() -> (String, String, String, String) {
    let key_name = String::from("id");
    let timestamp_name = String::from("ts");
    let from_table = String::from("the.matrix");
    let order_by = String::from("serial");

    (key_name, timestamp_name, from_table, order_by)
  }

  fn query_params_init() -> QueryParams {
    let (key_name, timestamp_name, from_table, order_by) = query_params();

    QueryParams { key_name,
                  timestamp_name,
                  from_table,
                  order_by }
  }

  fn source_init(rp: &RunParams, qp: &QueryParams) -> Source {
    Source { publishes:    stringvec!["lon", "lat", "speed"],
             run_params:   rp.clone(),
             query_params: qp.clone(),
             node_tx:      None, }
  }

  #[test]
  fn source_test() {
    let (starting_offset, millis_per_cycle, hours_to_run) = run_params();

    let rp = run_params_init();
    assert_eq!(rp.starting_offset, starting_offset);
    assert_eq!(rp.millis_per_cycle, millis_per_cycle);
    assert_eq!(rp.hours_to_run, hours_to_run);

    let (key_name, timestamp_name, from_table, order_by) = query_params();

    let qp = query_params_init();
    assert_eq!(qp.key_name, key_name);
    assert_eq!(qp.timestamp_name, timestamp_name);
    assert_eq!(qp.from_table, from_table);
    assert_eq!(qp.order_by, order_by);

    let src = source_init(&rp, &qp);
    assert_eq!(src.publishes, stringvec!["lon", "lat", "speed"]);
    assert_eq!(src.publishes(), stringvec!["lon", "lat", "speed"]);
    assert_eq!(src.subscribes_to(), Vec::<String>::new());
    assert_eq!(src.run_params, rp);
    assert_eq!(src.query_params, qp);
    assert!(src.node_tx.is_none());
  }
}
