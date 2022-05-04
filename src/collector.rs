// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::datapoint::DataPoint;

use eyre::Result;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
// use tokio_postgres::NoTls;
use tracing::info;

pub async fn start(mut linein: Receiver<DataPoint>) -> Result<JoinHandle<()>> {
  // let (db_client, connection) =
  //   tokio_postgres::connect("host=localhost user=postgres \
  //                            password=barbershop dbname=results",
  //                           NoTls).await?;
  // info!("database connection successful");

  let handle = tokio::spawn(async move {
    while let Some(dp) = linein.recv().await {
      info!("{}", dp);
    }
  });
  Ok(handle)
}
