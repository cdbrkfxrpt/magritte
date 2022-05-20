// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::SinkConfig, types::Message};

use tokio::sync::mpsc;
use tracing::info;


#[derive(Debug)]
pub struct Sink {
  config:  SinkConfig,
  sink_rx: mpsc::Receiver<Message>,
}

impl Sink {
  pub fn init(config: SinkConfig) -> (Self, mpsc::Sender<Message>) {
    let (sink_tx, sink_rx) = mpsc::channel(config.channel_capacity);
    info!("setup of Sink channel successful");

    (Self { config, sink_rx }, sink_tx)
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      while let Some(message) = self.sink_rx.recv().await {
        info!(?message);
      }
    });
  }
}
