// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::Config, types::FluentResult};

use indoc::indoc;
use tokio::sync::mpsc;
use tokio_postgres as tp;
use tracing::{error, info};


#[derive(Debug)]
pub struct Sink {
  config:  Config,
  sink_rx: mpsc::Receiver<FluentResult>,
}

impl Sink {
  pub fn init(config: Config) -> (Self, mpsc::Sender<FluentResult>) {
    let (sink_tx, sink_rx) = mpsc::channel(config.sink.channel_capacity);
    info!("setup of Sink channel successful");

    (Self { config, sink_rx }, sink_tx)
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      let dbparams = format!("host={} user={} password={} dbname={}",
                             self.config.database.host,
                             self.config.database.user,
                             self.config.database.password,
                             self.config.database.dbname);

      let (dbclient, connection) =
        tp::connect(&dbparams, tp::NoTls).await.unwrap();

      tokio::spawn(async move {
        if let Err(e) = connection.await {
          error!("connection error: {}", e);
        }
      });

      while let Some(fluent_result) = self.sink_rx.recv().await {
        info!(?fluent_result);
        dbclient.execute(
                         indoc! {r#"
            insert into
              magritte.results (
                source_id,
                timestamp,
                fluent_name,
                holds
              )
            values
              ($1, $2, $3, $4)
          "#},
                         &[
          &(fluent_result.source_id as i32),
          &(fluent_result.timestamp as i64),
          &fluent_result.name,
          &fluent_result.holds,
        ],
        )
                .await
                .unwrap();
      }
    });
  }
}
