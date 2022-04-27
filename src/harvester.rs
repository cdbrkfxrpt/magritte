// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::datapoint::DataPoint;

use eyre::Result;
use tokio::{task::JoinHandle, time};
use tokio_postgres::NoTls;
use tracing::{error, info};


const STEP_SIZE: i64 = 1;


pub struct Harvester {
  pub handle: JoinHandle<()>,
}

impl Harvester {
  pub async fn new() -> Result<Self> {
    let (db_client, connection) =
      tokio_postgres::connect("host=localhost user=postgres \
                               password=barbershop \
                               dbname=doi105281zenodo1167595",
                              NoTls).await?;
    info!("database connection successful");

    tokio::spawn(async move {
      if let Err(e) = connection.await {
        error!("connection error: {}", e);
      }
    });

    let statement = db_client.prepare("select * from ais_data.dynamic_ships \
                                       order by id asc offset $1 rows fetch \
                                       next 32 rows only")
                             .await?;
    info!("SQL statement prepared");

    let handle = tokio::spawn(async move {
      let mut interval =
        time::interval(time::Duration::from_secs(STEP_SIZE as u64));

      let mut time: i64 = 1443650400;
      let mut offset: i64 = 0;

      loop {
        interval.tick().await;
        let rows = db_client.query(&statement, &[&(offset)]).await.unwrap();

        for row in rows {
          let dp = DataPoint::from_row(row);
          if dp.ts() <= &time {
            info!("processing {}", dp);
            offset += 1;
          }
        }

        time += STEP_SIZE;
      }
    });

    Ok(Self { handle })
  }

  pub fn abort(&mut self) {
    self.handle.abort();
  }
}
