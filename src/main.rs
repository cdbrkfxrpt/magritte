// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

//! `magritte` - _Ceci n'est pas une pipe(line)_.
//!
//! This application implements the concurrent stream reasoning architecture
//! outline in the Master's thesis of Florian Eich.

// mod collector;
mod config;
// mod datapoint;
// mod dispatcher;
// mod feeder;

use config::Config;

use clap::Parser;
use eyre::Result;
// use std::time;
use tokio::signal;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};


#[tokio::main]
async fn main() -> Result<()> {
  setup()?;

  info!("logging and tracing setup complete, reading comand line parameters");
  let conf = Config::parse();

  info!("CONFIGURATION REPORT: \n{:#?}", conf);

  info!("all ready for take-off, magritte starting up");
  let magritte = {
    let context = Context::new()?;

    let feeder = Feeder::new(context)?.with_step(conf.millis_per_step)?;
    let dispatcher = Dispatcher::new(context)?;
    let collector = Collector::new(context)?;

    Core::builder().add_feeder(feeder)
                   .add_dispatcher(dispatcher)
                   .add_collector(collector)
                   .run()
  };

  // let dispatcher = Dispatcher::new()
  //
  // .add_feeder(Feeder::with_cycle_time(time::Duration::from_secs(1))?)
  //                   .add_collector(Collector::new()?)?;

  // let feeder = ;

  // let (feeder, lineout) = feeder::start().await?;
  // let (dispatcher, lineout) = dispatcher::start(lineout).await?;
  // let collector = collector::start(lineout).await?;

  info!("magritte task has been started, setting up Ctrl+C listener");
  signal::ctrl_c().await?;

  info!("magritte has received Ctrl+C, shutting down...");
  magritte.abort();

  info!("magritte has finished");
  Ok(())
}

/// Initalizes backtracing and error handling capabilities and sets up the
/// tracing infrastructure for outputting logs from all components as well
/// monitoring tasks through tokio console.
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

// fin --------------------------------------------------------------------- //
