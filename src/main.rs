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
use tracing_subscriber::{fmt, prelude::*, EnvFilter};


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
  // set up eyre with colors
  const BT_ENVVAR: &str = "RUST_LIB_BACKTRACE";
  if std::env::var(BT_ENVVAR).is_err() {
    std::env::set_var(BT_ENVVAR, "1")
  }
  color_eyre::install()?;

  // set up console layer for tracing
  let console_layer = console_subscriber::spawn();

  // set up format layer with filtering for tracing
  const LG_ENVVAR: &str = "RUST_LOG";
  if std::env::var(LG_ENVVAR).is_err() {
    std::env::set_var(LG_ENVVAR, "info")
  }
  let format_layer = fmt::layer().with_filter(EnvFilter::from_default_env());

  // bring the tracing subscriber together with both layers
  tracing_subscriber::registry().with(console_layer)
                                .with(format_layer)
                                .init();

  Ok(())
}
