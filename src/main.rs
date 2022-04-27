// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

mod datapoint;
mod harvester;

use harvester::Harvester;

use eyre::Result;
use tokio::signal;
use tracing::info;
use tracing_subscriber::EnvFilter;


#[tokio::main]
async fn main() -> Result<()> {
  setup()?;
  info!("logging and tracing setup complete, magritte starting up");

  let mut harvester = Harvester::new().await?;

  signal::ctrl_c().await?;

  info!("magritte has received Ctrl+C, shutting down...");
  harvester.abort();

  info!("magritte has finished");
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
