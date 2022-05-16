// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::FeederConfig, types::Datapoint};

use indoc::indoc;
use tokio::{sync::mpsc, task::JoinHandle, time};
use tokio_postgres::NoTls;
use tracing::{error, info};


#[derive(Debug)]
pub struct Feeder {
  config: FeederConfig,
  out_tx: mpsc::Sender<Datapoint>,
}


impl Feeder {
  pub fn init(config: FeederConfig) -> (Self, mpsc::Receiver<Datapoint>) {
    let (out_tx, out_rx) = mpsc::channel(config.channel_capacity);
    info!("setup of Feeder channel successful");

    (Self { config, out_tx }, out_rx)
  }

  pub fn run(self) -> JoinHandle<()> {
    tokio::spawn(async move {
      let dbparams = format!("host={} user={} password={} dbname={}",
                             self.config.connection.host,
                             self.config.connection.user,
                             self.config.connection.password,
                             self.config.connection.dbname);

      let (db_client, connection) =
        tokio_postgres::connect(&dbparams, NoTls).await.unwrap();

      info!("database connection successful");

      // task awaits database connection, traces on error
      tokio::spawn(async move {
        if let Err(e) = connection.await {
          error!("connection error: {}", e);
        }
      });

      let statement_raw = format!(
                                  indoc! {r#"
          select {} as "source_id", {} as "timestamp", {}
          from {}
          order by {} asc
          offset $1 rows
          fetch next {} rows only
        "#},
                                  self.config.query.source_id,
                                  self.config.query.timestamp,
                                  self.config.query.value_names.join(", "),
                                  self.config.query.from_table,
                                  self.config.query.order_by,
                                  self.config.channel_capacity
      );
      info!(?statement_raw);

      let statement = match db_client.prepare(&statement_raw).await {
        Ok(statement) => statement,
        Err(err) => {
          drop(self.out_tx);
          panic!("Error in Feeder PostgreSQL query: {}", err);
        }
      };
      info!("SQL statement prepared");

      let mut interval =
        time::interval(time::Duration::from_millis(self.config
                                                       .millis_per_cycle));

      let mut time: usize = 1443650400;
      let mut offset: i64 = 0;
      let mut datapoint = Datapoint::new(&self.config.query.value_names);

      loop {
        interval.tick().await;
        let rows = db_client.query(&statement, &[&(offset)]).await.unwrap();

        for row in rows {
          datapoint.update(row);
          if datapoint.timestamp <= time {
            // TODO: fix unwrap
            self.out_tx.send(datapoint.clone()).await.unwrap();
            //
            offset += 1;
          }
        }
        time += 1;
      }
    })
  }
}

// TODO: write tests
//
