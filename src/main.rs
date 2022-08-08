// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

//! `magritte` - _Ceci n'est pas une pipe(line)_
//!
//! An application for concurrently reasoning over data streams. Configurably
//! reads data from a PostgreSQL database (other input interfaces are possible
//! by extending the application, which is well-documented and straightforward)
//! and applies rules provided by the user on it. The resulting _event stream_
//! is written back to the PostgreSQL database (although, again, this can
//! easily be adjusted).
//!
//! Theoretical background is provided in the form of [a graduation thesis for
//! M.Sc. in Computer
//! Science](https://gitlab.bmc-labs.com/flrn/eich21/-/raw/trunk/thesis/99-PDF/eich21.pdf?inline=false)
//! by [Florian Eich](mailto:florian.eich@gmail.com).

// crate level attributes
#![allow(dead_code)] // remove once done
#![feature(mutex_unlock)]
#![feature(box_into_inner)]
#![feature(let_else)]
//

mod app_init;
mod broker;
mod database;
mod fluent;
mod source;
mod util;

use app_init::AppInit;

use eyre::Result;
use tokio::{signal, sync::mpsc};
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};


#[derive(Debug)]
enum ShutdownCause {
  BrokerShutdown,
  BrokerInitFailed,
  CtrlC,
}


#[tokio::main]
/// Application entry point. Sets up core services and runs the application.
async fn main() -> Result<()> {
  setup()?;
  info!("logging and tracing setup complete, magritte starting up");

  // this channel is used by service tasks communicate back to main
  let (tx, mut rx) = mpsc::unbounded_channel();

  info!("Ctrl+C listener starting up...");
  let main_tx = tx.clone();
  tokio::spawn(async move {
    signal::ctrl_c().await
                    .expect("unable to listen for Ctrl+C event");

    info!("received Ctrl+C signal");
    if let Err(e) = main_tx.send(ShutdownCause::CtrlC) {
      error!("unable to inform magritte main task: {}", e);
    }
  });

  info!("reading command line arguments and config file to init app...");
  let AppInit { database_connector,
                source,
                broker, } = AppInit::parse()?;
  info!(?database_connector, ?source, ?broker);

  info!("preparing database for run...");
  database_connector.prepare_run().await?;

  // broker.register_source(source);
  // broker.register_sink(sink);
  // for node in build_node_index() {
  //   broker.register_node(node)?;
  // }

  let main_tx = tx.clone();
  let broker_task = tokio::spawn(async move {
    match broker.run().await {
      Ok(()) => {
        info!("broker has stopped");
        if let Err(e) = main_tx.send(ShutdownCause::BrokerShutdown) {
          error!("unable to inform magritte main task: {}", e);
        }
      }
      Err(e) => {
        info!("broker init failed: {}", e);
        if let Err(e) = main_tx.send(ShutdownCause::BrokerInitFailed) {
          error!("unable to inform magritte main task: {}", e);
        }
      }
    }
  });

  info!("magritte up and running!");
  match rx.recv()
          .await
          .expect("received None on magritte main task channel")
  {
    ShutdownCause::BrokerShutdown | ShutdownCause::BrokerInitFailed => (),
    ShutdownCause::CtrlC => broker_task.abort(),
  }

  info!("magritte has shut down");
  Ok(())
}

/// Initalizes backtracing and error handling capabilities. Sets up tracing and
/// task monitoring through tokio console.
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
