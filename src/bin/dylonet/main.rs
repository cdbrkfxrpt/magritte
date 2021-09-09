// Copyright 2021 bmc::labs GmbH.
//
// Authors:
//   Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use magritte::dylonet_service_server::DylonetServiceServer;

use eyre::Result;
use tokio::signal;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

mod dylonet;
mod handler;


#[tokio::main]
async fn main() -> Result<()> {
  setup()?;
  info!("logging and tracing setup complete, dylonet starting up");

  let location = "127.0.0.1:52525".parse()?;
  let server = tokio::spawn(async move {
    let dylonet_server = DylonetServiceServer::new(handler::Handler::new());
    Server::builder().add_service(dylonet_server)
                     .serve(location)
                     .await
  });

  signal::ctrl_c().await?;

  info!("dylonet has received Ctrl+C, shutting down...");
  server.abort();

  info!("dylonet has finished");
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
  fmt::fmt().with_env_filter(EnvFilter::from_default_env())
            .init();

  Ok(())
}
