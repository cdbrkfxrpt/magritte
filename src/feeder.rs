// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::{DatabaseCredentials, FeederConfig},
            types::Datapoint};

use indoc::indoc;
use tokio::{sync::mpsc, task::JoinHandle, time};
use tokio_postgres as tp;
use tracing::{error, info};


#[derive(Debug)]
pub struct Feeder {
  data_tx:              mpsc::Sender<Datapoint>,
  database_credentials: DatabaseCredentials,
  feeder_config:        FeederConfig,
}


impl Feeder {
  pub fn init(data_tx: mpsc::Sender<Datapoint>,
              database_credentials: &DatabaseCredentials,
              feeder_config: &FeederConfig)
              -> Self {
    let database_credentials = database_credentials.clone();
    let feeder_config = feeder_config.clone();

    Self { data_tx,
           database_credentials,
           feeder_config }
  }

  pub fn run(self) -> JoinHandle<()> {
    tokio::spawn(async move {
      let dbparams = format!("host={} user={} password={} dbname={}",
                             self.database_credentials.host,
                             self.database_credentials.user,
                             self.database_credentials.password,
                             self.database_credentials.dbname);

      let (dbclient, connection) =
        tp::connect(&dbparams, tp::NoTls).await.unwrap();

      info!("database connection successful");

      // task awaits database connection, traces on error
      tokio::spawn(async move {
        if let Err(e) = connection.await {
          error!("connection error: {}", e);
        }
      });

      let statement_raw =
        format!(
                indoc! {r#"
          select {} as "source_id", {} as "timestamp", {}
          from {}
          order by {} asc
          offset $1 rows
          fetch next {} rows only
        "#},
                self.feeder_config.query.source_id,
                self.feeder_config.query.timestamp,
                self.feeder_config.query.value_names.join(", "),
                self.feeder_config.query.from_table,
                self.feeder_config.query.order_by,
                self.data_tx.capacity()
        );

      let statement = match dbclient.prepare(&statement_raw).await {
        Ok(statement) => statement,
        Err(err) => {
          // drop(self.data_tx);
          panic!("Error in Feeder PostgreSQL query: {}", err);
        }
      };
      info!("SQL statement prepared");

      let mut interval =
        time::interval(time::Duration::from_millis(self.feeder_config
                                                       .millis_per_cycle));

      let mut time: usize = 1443650400;
      let mut offset: usize = 0;
      let mut datapoint =
        Datapoint::new_datapoint(0, 0, &self.feeder_config.query.value_names);

      loop {
        interval.tick().await;
        let rows = dbclient.query(&statement, &[&(offset as i64)])
                           .await
                           .unwrap();

        for row in rows {
          datapoint.update_datapoint(row);
          if datapoint.timestamp <= time {
            // info!("data_tx capacity: {}", self.data_tx.capacity());
            // TODO: fix unwrap
            self.data_tx.send(datapoint.clone()).await.unwrap();
            //
            offset += 1;
          }
          // if offset == self.config.feeder.datapoints_to_run {
          //   return ();
          // }
        }
        time += 1;
        // info!("ran {} datapoints", offset);
      }
    })
  }
}

// TODO: write tests
//
