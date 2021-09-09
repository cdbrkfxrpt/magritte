// Copyright 2021 bmc::labs GmbH.
//
// Authors:
//   Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use magritte::{dylonet_service_client::DylonetServiceClient,
               DataPoint,
               Empty,
               State};

use eyre::Result;
use tokio::{signal, time};
use tokio_postgres::NoTls;
use tonic::{transport::Channel, Request};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;


const STEP_SIZE: i64 = 1;


#[tokio::main]
async fn main() -> Result<()> {
  setup()?;
  info!("logging and tracing setup complete, harvester starting up");

  let mut dylonet_client =
    match DylonetServiceClient::connect("http://127.0.0.1:52525").await {
      Ok(client) => client,
      Err(_) => {
        error!("unable to connect to dylonet, aborting");
        return Ok(());
      }
    };

  let mut state = get_state(&mut dylonet_client).await?;
  while state != State::Ready {
    info!("dylonet state: {:?}", state);
    time::sleep(time::Duration::from_secs(3)).await;
    state = get_state(&mut dylonet_client).await?;
  }
  info!("dylonet is ready");

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

  let client_task = tokio::spawn(async move {
    let mut interval =
      time::interval(time::Duration::from_secs(STEP_SIZE as u64));

    let mut time: i64 = 1443650400;
    let mut offset: i64 = 0;

    loop {
      interval.tick().await;
      let rows = db_client.query(&statement, &[&(offset)]).await.unwrap();

      for row in rows {
        let dp = DataPoint::from_row(row);
        if dp.ts <= time {
          info!("processing {}", dp);
          let dylonet_response = dylonet_client.deliver(Request::new(dp))
                                               .await
                                               .unwrap()
                                               .into_inner();
          info!("dylonet response: {:?}", dylonet_response);
          offset += 1;
        }
      }

      time += STEP_SIZE;
    }
  });

  signal::ctrl_c().await?;

  info!("harvester has received Ctrl+C, shutting down...");
  client_task.abort();

  info!("harvester has finished");
  Ok(())
}

fn setup() -> Result<()> {
  const BT_ENVVAR: &str = "RUST_LIB_BACKTRACE";
  if std::env::var(BT_ENVVAR).is_err() {
    std::env::set_var(BT_ENVVAR, "1")
  }
  color_eyre::install()?;

  const LG_ENVVAR: &str = "RUST_LOG";
  if std::env::var(LG_ENVVAR).is_err() {
    std::env::set_var(LG_ENVVAR, "info")
  }
  tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

  Ok(())
}

async fn get_state(client: &mut DylonetServiceClient<Channel>)
                   -> Result<State> {
  Ok(State::from(client.state(Request::new(Empty {}))
                       .await?
                       .into_inner()
                       .state))
}
