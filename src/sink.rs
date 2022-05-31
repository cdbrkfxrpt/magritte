// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::Config,
            types::{Message, RuleResult}};

use indoc::indoc;
use tokio::sync::mpsc;
use tokio_postgres as tp;
use tracing::{error, info};


#[derive(Debug)]
pub struct Sink {
  config:  Config,
  sink_rx: mpsc::Receiver<Message>,
}

impl Sink {
  pub fn init(config: Config) -> (Self, mpsc::Sender<Message>) {
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

      while let Some(message) = self.sink_rx.recv().await {
        info!(?message);
        let Message::Response { fn_name,
                                source_id,
                                timestamp,
                                values,
                                rule_result } = message else {
          panic!("not a response");
        };

        let RuleResult::Boolean(rule_result) = rule_result else {
          panic!("rule_result is not boolean");
        };

        dbclient.execute(
                         indoc! {r#"
            insert into
              magritte.results (
                name,
                source_id,
                timestamp,
                rule_result,
                lon,
                lat,
                speed
              )
            values
              ($1, $2, $3, $4, $5, $6, $7)
          "#},
                         &[
          &fn_name,
          &(source_id as i32),
          &(timestamp as i64),
          &rule_result,
          &values["lon"],
          &values["lat"],
          &values["speed"],
        ],
        )
                .await
                .unwrap();
      }
    });
  }
}
