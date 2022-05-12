// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::Config, datapoint::DataPoint};

use eyre::Result;
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

  pub async fn run(self) -> Result<()> {
    let (db_client, connection) =
      tokio_postgres::connect(indoc! {"
                              host=localhost \
                              user=postgres \
                              password=barbershop \
                              dbname=doi105281zenodo1167595"},
                              NoTls).await?;
    info!("database connection successful");

    // task awaits database connection, traces on error
    tokio::spawn(async move {
      if let Err(e) = connection.await {
        error!("connection error: {}", e);
      }
    });

    let statement_raw =
      format!(include_str!("../sql/feeder.sql"), self.channel_capacity);
    let statement = db_client.prepare(&statement_raw).await?;
    info!("SQL statement prepared");

    tokio::spawn(async move {
      let mut interval =
        time::interval(time::Duration::from_millis(self.millis_per_cycle
                                                   as u64));

      let mut time: i64 = 1443650400;
      let mut offset: i64 = 0;

      loop {
        interval.tick().await;
        let rows = db_client.query(&statement,
                                   &[&(offset),
                                     &(self.channel_capacity as i64)])
                            .await
                            .unwrap();

        for row in rows {
          let dp = DataPoint::from_row(row);
          if dp.timestamp() <= &time {
            info!("{}", dp);
            // TODO: fix unwrap
            self.to_receiver.send(dp).await.unwrap();
            //
            offset += 1;
          }
        }

        time += self.millis_per_cycle as i64;
      }
    });

    Ok(())
  }
}

// TODO: write tests
//
