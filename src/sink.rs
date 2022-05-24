// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::SinkConfig,
            types::{Message, RuleResult}};

use indoc::indoc;
use tokio::sync::mpsc;
use tokio_postgres as tp;
use tracing::{error, info};


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
      let dbparams = format!("host={} user={} password={} dbname={}",
                             self.config.connection.host,
                             self.config.connection.user,
                             self.config.connection.password,
                             self.config.connection.dbname);

      let (dbclient, connection) =
        tp::connect(&dbparams, tp::NoTls).await.unwrap();

      tokio::spawn(async move {
        if let Err(e) = connection.await {
          error!("connection error: {}", e);
        }
      });

      dbclient.batch_execute(indoc! {r#"
        drop schema if exists magritte cascade;
        create schema magritte;
        create table magritte.results (
          name        text,
          source_id   serial,
          timestamp   bigint,
          rule_result bool,
          lon         double precision,
          lat         double precision,
          speed       double precision
        );
        "#})
              .await
              .unwrap();

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
