// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::Config, datapoint::DataPoint};

use indoc::indoc;
use tokio::{sync::{mpsc,
                   mpsc::{Receiver, Sender}},
            time};
use tokio_postgres::NoTls;
use tracing::{error, info};


#[derive(Debug)]
pub struct Feeder {
  channel_capacity: usize,
  millis_per_cycle: usize,
  to_receiver:      Sender<DataPoint>,
}


impl Feeder {
  pub fn init(config: &Config) -> (Self, Receiver<DataPoint>) {
    let (to_receiver, receiver) = mpsc::channel(config.channel_capacity);
    info!("setup of Feeder channel successful");

    (Self { channel_capacity: config.channel_capacity,
            millis_per_cycle: config.millis_per_cycle,
            to_receiver },
     receiver)
  }

  pub fn run(self) {
    tokio::spawn(async move {
      let (db_client, connection) =
        tokio_postgres::connect(indoc! {"
                                host=localhost \
                                user=postgres \
                                password=barbershop \
                                dbname=doi105281zenodo1167595"},
                                NoTls).await
                                      .unwrap();
      info!("database connection successful");

      // task awaits database connection, traces on error
      tokio::spawn(async move {
        if let Err(e) = connection.await {
          error!("connection error: {}", e);
        }
      });

      let statement_raw =
        format!(include_str!("../sql/feeder.sql"), self.channel_capacity);
      info!(?statement_raw);

      let statement = match db_client.prepare(&statement_raw).await {
        Ok(statement) => statement,
        Err(err) => {
          drop(self.to_receiver);
          panic!("Error in Feeder PostgreSQL query: {}", err);
        }
      };
      info!("SQL statement prepared");

      let mut interval =
        time::interval(time::Duration::from_millis(self.millis_per_cycle
                                                   as u64));

      let mut time: i64 = 1443650400;
      let mut offset: i64 = 0;

      loop {
        interval.tick().await;
        let rows = db_client.query(&statement, &[&(offset)]).await.unwrap();

        for row in rows {
          let dp = DataPoint::from_row(row);
          if dp.timestamp() <= &time {
            // TODO: fix unwrap
            self.to_receiver.send(dp).await.unwrap();
            //
            offset += 1;
          }
        }
        time += 1;
      }
    });
  }
}

// TODO: write tests
//
