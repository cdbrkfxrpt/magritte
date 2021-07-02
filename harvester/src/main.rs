// Copyright 2021 bmc::labs GmbH.
//
// Authors:
//   Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use tokio::time;
use tokio_postgres::{Error, NoTls};

mod data_point;
use data_point::DataPoint;


#[tokio::main]
async fn main() -> Result<(), Error> {
  let (client, connection) =
    tokio_postgres::connect("host=localhost user=postgres \
                             password=barbershop \
                             dbname=doi105281zenodo1167595",
                            NoTls).await?;

  tokio::spawn(async move {
    if let Err(e) = connection.await {
      eprintln!("connection error: {}", e);
    }
  });

  let statement = client.prepare("select * from ais_data.dynamic_ships \
                                  order by id asc offset $1 rows fetch next \
                                  32 rows only")
                        .await?;

  let step_size: i64 = 1;
  let mut interval =
    time::interval(time::Duration::from_millis(step_size as u64 * 1000));

  let mut time: i64 = 1443650400;
  let mut offset: i64 = 0;

  loop {
    interval.tick().await;
    let rows = client.query(&statement, &[&offset]).await?;

    for row in rows {
      let dp = DataPoint::from_row(row);
      if dp.ts() <= &time {
        println!("{}", dp);
        offset += 1;
      }
    }

    time += step_size;
  }

  // Ok(())
}
