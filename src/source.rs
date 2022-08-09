// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::fluent::AnyFluent;

use eyre::Result;
use indoc::indoc;
use serde::Deserialize;
use tokio::{sync::mpsc, time};
use tokio_postgres::Client;
use tracing::info;


#[derive(Clone, Debug, PartialEq, Deserialize)]
/// Holds parameters for the `Source` service of the application, which reads
/// data from the source (i.e. the PostgreSQL database) and publishes it to the
/// `Broker` service.
pub struct Source {
  run_params:   RunParams,
  query_params: QueryParams,
}

impl Source {
  pub fn published_fluents(&self) -> Vec<String> {
    self.query_params.fluent_names.clone()
  }

  pub async fn run(self,
                   database_client: Client,
                   data_tx: mpsc::UnboundedSender<AnyFluent>)
                   -> Result<()> {
    let statement_raw = format!(
                                indoc! {r#"
          select {} as "key", {} as "timestamp", {}
          from {}
          order by {} asc
          offset $1 rows
          fetch next {} rows only
        "#},
                                self.query_params.key_name,
                                self.query_params.timestamp_name,
                                self.query_params.fluent_names.join(", "),
                                self.query_params.from_table,
                                self.query_params.order_by,
                                self.query_params.rows_to_fetch
    );

    let statement = match database_client.prepare(&statement_raw).await {
      Ok(statement) => statement,
      Err(err) => {
        // drop(self.data_tx);
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
          for fluent_name in self.query_params.fluent_names.iter() {
            let value: f64 = row.get(fluent_name.as_str());
            let fluent =
              AnyFluent::new(&fluent_name, &[key], timestamp, value);

            data_tx.send(fluent)?;
          }
          offset += 1;
        }
      }
      time += 1;
      info!("ran {} datapoints", offset);

      interval.tick().await;
    }

    Ok(())
  }
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
  pub rows_to_fetch:  usize,
}


// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::{QueryParams, RunParams, Source};
  use crate::{app_init::AppInit, fluent::AnyFluent, util::stringvec};

  use pretty_assertions::assert_eq;
  use tokio::sync::mpsc;


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
    let fluent_names = stringvec!["lon", "lat", "speed"];
    let from_table = String::from("the.matrix");
    let order_by = String::from("serial");
    let rows_to_fetch = 32;

    let qp = QueryParams { key_name:       key_name.clone(),
                           timestamp_name: timestamp_name.clone(),
                           fluent_names:   fluent_names.clone(),
                           from_table:     from_table.clone(),
                           order_by:       order_by.clone(),
                           rows_to_fetch:  rows_to_fetch, };

    assert_eq!(qp.key_name, key_name);
    assert_eq!(qp.timestamp_name, timestamp_name);
    assert_eq!(qp.fluent_names, fluent_names);
    assert_eq!(qp.from_table, from_table);
    assert_eq!(qp.order_by, order_by);
    assert_eq!(qp.rows_to_fetch, rows_to_fetch);

    let src = Source { run_params:   rp.clone(),
                       query_params: qp.clone(), };

    assert_eq!(src.run_params, rp);
    assert_eq!(src.query_params, qp);
  }

  #[tokio::test]
  // #[ignore]
  async fn source_test() {
    let app_init = AppInit::parse().unwrap();

    let database_client = app_init.database_connector.connect().await.unwrap();
    let source = app_init.source;

    assert_eq!(source.published_fluents(),
               stringvec!["lon", "lat", "speed"]);

    let (tx, mut rx) = mpsc::unbounded_channel();

    let runner = tokio::spawn(async move {
      source.run(database_client, tx).await.unwrap();
    });

    let AnyFluent::FloatPt(fluent) = rx.recv().await.unwrap() else {
      panic!()
    };

    assert_eq!(fluent.name(), "lon");
    assert_eq!(fluent.keys(), &[245257000]);
    assert_eq!(fluent.timestamp(), 1443650402);
    assert_eq!(fluent.value(), &-4.4657183);
    assert_eq!(fluent.last_change(), 1443650402);

    runner.abort();
  }
}
