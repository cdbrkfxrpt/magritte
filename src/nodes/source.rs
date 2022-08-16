// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Node, NodeRx, NodeTx, StructuralNode};
use crate::fluent::AnyFluent;

use async_trait::async_trait;
use eyre::{bail, Result};
use serde::Deserialize;
use tokio::time;
use tokio_postgres::Client;
use tracing::info;


#[derive(Debug, Deserialize)]
/// Reads data from the source (i.e. the PostgreSQL database) and publishes it
/// to the [`Broker`](crate::app_core::Broker) service.
pub struct Source {
  publishes:    Vec<String>,
  run_params:   RunParams,
  query_params: QueryParams,
  #[serde(skip)]
  node_tx:      Option<NodeTx>,
}

impl Node for Source {
  /// `Source` publishes fluents specified by name in the app configuration.
  fn publishes(&self) -> Vec<String> {
    self.publishes.clone()
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

#[async_trait]
impl StructuralNode for Source {
  /// Runs the [`Source`], retrieving data from the database and publishing
  /// fluents to the [`Broker`](crate::app_core::Broker). Consumes the original
  /// object.
  ///
  /// Data retrieval is performed using the following SQL:
  ///
  /// ```sql
  #[doc = include_str!("../sql/source.sql")]
  /// ```
  async fn run(mut self: Box<Self>, database_client: Client) -> Result<()> {
    let node_tx = match self.node_tx {
      Some(node_tx) => node_tx,
      None => bail!("Source not initialized, aborting"),
    };

    let statement_raw =
      format!(include_str!("../sql/source.sql"),
              key_name = self.query_params.key_name,
              timestamp_name = self.query_params.timestamp_name,
              fluent_names = self.publishes.join(", "),
              from_table = self.query_params.from_table,
              order_by = self.query_params.order_by,
              rows_to_fetch = self.query_params.rows_to_fetch);

    let statement = match database_client.prepare(&statement_raw).await {
      Ok(statement) => statement,
      Err(err) => {
        // drop(self.fluent_tx);
        panic!("Error in Feeder PostgreSQL query: {}", err);
      }
    };
    info!("SQL statement prepared");

    let mut interval =
      time::interval(time::Duration::from_millis(self.run_params
                                                     .millis_per_cycle));

    let mut time: usize = 1443650400;
    let mut offset: usize = 0;

    let mut key: usize;
    let mut timestamp: usize;

    while let Ok(rows) =
      database_client.query(&statement, &[&(offset as i64)]).await
    {
      for row in rows {
        key = row.get::<&str, i32>("key") as usize;
        timestamp = row.get::<&str, i64>("timestamp") as usize;

        if timestamp <= time {
          for fluent_name in self.publishes.iter() {
            let value: f64 = row.get(fluent_name.as_str());
            let fluent =
              AnyFluent::new(&fluent_name, &[key], timestamp, Box::new(value));

            node_tx.send(fluent)?;
          }
          offset += 1;

          info!("ran {} datapoints", offset);
          if self.run_params.datapoints_to_run > 0
             && offset == self.run_params.datapoints_to_run
          {
            info!("reached limit of datapoints to run ({})",
                  self.run_params.datapoints_to_run);
            return Ok(());
          }
        }
      }
      time += 1;

      interval.tick().await;
    }

    Ok(())
  }
}


#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Holds parameters for the execution of the [`Source`] service.
struct RunParams {
  pub millis_per_cycle:  u64,
  pub datapoints_to_run: usize,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Holds parameters for the database query performed by the [`Source`].
struct QueryParams {
  pub key_name:       String,
  pub timestamp_name: String,
  pub from_table:     String,
  pub order_by:       String,
  pub rows_to_fetch:  usize,
}


// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::{Node, QueryParams, RunParams, Source};
  use crate::stringvec;

  use pretty_assertions::assert_eq;


  fn run_params() -> (u64, usize) {
    let millis_per_cycle = 42;
    let datapoints_to_run = 1337;

    (millis_per_cycle, datapoints_to_run)
  }

  fn run_params_init() -> RunParams {
    let (millis_per_cycle, datapoints_to_run) = run_params();

    RunParams { millis_per_cycle,
                datapoints_to_run }
  }

  fn query_params() -> (String, String, String, String, usize) {
    let key_name = String::from("id");
    let timestamp_name = String::from("ts");
    let from_table = String::from("the.matrix");
    let order_by = String::from("serial");
    let rows_to_fetch = 32;

    (key_name, timestamp_name, from_table, order_by, rows_to_fetch)
  }

  fn query_params_init() -> QueryParams {
    let (key_name, timestamp_name, from_table, order_by, rows_to_fetch) =
      query_params();

    QueryParams { key_name,
                  timestamp_name,
                  from_table,
                  order_by,
                  rows_to_fetch }
  }

  fn source_init(rp: &RunParams, qp: &QueryParams) -> Source {
    Source { publishes:    stringvec!["lon", "lat", "speed"],
             run_params:   rp.clone(),
             query_params: qp.clone(),
             node_tx:      None, }
  }

  #[test]
  fn source_test() {
    let (millis_per_cycle, datapoints_to_run) = run_params();

    let rp = run_params_init();
    assert_eq!(rp.millis_per_cycle, millis_per_cycle);
    assert_eq!(rp.datapoints_to_run, datapoints_to_run);

    let (key_name, timestamp_name, from_table, order_by, rows_to_fetch) =
      query_params();

    let qp = query_params_init();
    assert_eq!(qp.key_name, key_name);
    assert_eq!(qp.timestamp_name, timestamp_name);
    assert_eq!(qp.from_table, from_table);
    assert_eq!(qp.order_by, order_by);
    assert_eq!(qp.rows_to_fetch, rows_to_fetch);

    let src = source_init(&rp, &qp);
    assert_eq!(src.publishes, stringvec!["lon", "lat", "speed"]);
    assert_eq!(src.publishes(), stringvec!["lon", "lat", "speed"]);
    assert_eq!(src.subscribes_to(), Vec::<String>::new());
    assert_eq!(src.run_params, rp);
    assert_eq!(src.query_params, qp);
    assert!(src.node_tx.is_none());
  }
}
