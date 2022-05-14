// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

//! `magritte` - _Ceci n'est pas une pipe(line)_.
//!
//! This application implements the concurrent stream reasoning architecture
//! outline in the Master's thesis of Florian Eich.

// crate level attributes
#![allow(dead_code)] // remove once done
//

mod config;
mod datapoint;
mod feeder;
mod magritte;

use magritte::Magritte;

use eyre::Result;
use tokio::{signal, sync::mpsc};
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};


#[derive(Debug)]
enum AbortSource {
  Internal,
  External,
}


#[tokio::main]
async fn main() -> Result<()> {
  setup()?;
  info!("logging and tracing setup complete, magritte starting up");

  let (tx, mut rx) = mpsc::unbounded_channel();

  let magritte_tx = tx.clone();
  let magritte = tokio::spawn(async move {
    Magritte::new().run().await.expect("unable to await runner");

    info!("runner has stopped");
    match magritte_tx.send(AbortSource::Internal) {
      Ok(()) => (),
      Err(e) => error!("unable to inform magritte main task: {}", e),
    }
  });

  info!("magritte runner has been started, setting up Ctrl+C listener");
  let ctrl_c_tx = tx.clone();
  tokio::spawn(async move {
    signal::ctrl_c().await
                    .expect("unable to listen for Ctrl+C event");

    info!("received Ctrl+C signal");
    match ctrl_c_tx.send(AbortSource::External) {
      Ok(()) => (),
      Err(e) => {
        error!("unable to inform magritte main task: {}", e)
      }
    }
  });

  match rx.recv()
          .await
          .expect("received None on magritte main task channel")
  {
    AbortSource::Internal => (),
    AbortSource::External => magritte.abort(),
  }

  info!("magritte has shut down");
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
