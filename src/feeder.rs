// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::datapoint::DataPoint;

use eyre::Result;
use tokio::{sync::{mpsc, mpsc::Receiver},
            task::JoinHandle,
            time};
use tokio_postgres::NoTls;
use tracing::{error, info};


const STEP_SIZE: i64 = 1;
const CAPACITY: usize = 32;


pub async fn start() -> Result<(JoinHandle<()>, Receiver<DataPoint>)> {
  let (db_client, connection) =
    tokio_postgres::connect("host=localhost user=postgres \
                             password=barbershop \
                             dbname=doi105281zenodo1167595",
                            NoTls).await?;
  info!("database connection successful");

  let (tx, lineout) = mpsc::channel(CAPACITY);

  // all this does is wait for the database connection to fail and spit out an
  // error if that happens
  tokio::spawn(async move {
    if let Err(e) = connection.await {
      error!("connection error: {}", e);
    }
  });

  let statement = db_client.prepare("select * from ais_data.dynamic_ships \
                                     order by id asc offset $1 rows fetch \
                                     next $2 rows only")
                           .await?;
  info!("SQL statement prepared");

  let handle = tokio::spawn(async move {
    let mut interval =
      time::interval(time::Duration::from_secs(STEP_SIZE as u64));
      // time::interval(time::Duration::from_millis(STEP_SIZE as u64));

    let mut time: i64 = 1443650400;
    let mut offset: i64 = 0;

    loop {
      interval.tick().await;
      let rows = db_client.query(&statement, &[&(offset), &(CAPACITY as i64)])
                          .await
                          .unwrap();

      for row in rows {
        let dp = DataPoint::from_row(row);
        if dp.timestamp() <= &time {
          info!("{}", dp);
          // TODO: fix unwrap
          tx.send(dp).await.unwrap();
          //
          offset += 1;
        }
      }

      time += STEP_SIZE;
    }
  });

  Ok((handle, lineout))
}


// TODO: write tests
//
