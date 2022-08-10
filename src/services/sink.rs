// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::fluent::AnyFluent;

use eyre::Result;
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_postgres::Client;
// use tracing::info;


#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Sink {}

impl Sink {
  pub async fn run(self,
                   database_client: Client,
                   mut fluent_rx: mpsc::UnboundedReceiver<AnyFluent>)
                   -> Result<()> {
    let sql_raw = include_str!("sink.sql");

    while let Some(fluent) = fluent_rx.recv().await {
      if let AnyFluent::Boolean(fluent) = fluent {
        let name = fluent.name();
        let keys = fluent.keys().iter().map(|&e| e as i32).collect::<Vec<_>>();
        let timestamp = fluent.timestamp() as i64;
        let value = fluent.value();
        let last_change = fluent.last_change() as i64;

        database_client.execute(sql_raw,
                                &[&name,
                                  &keys,
                                  &timestamp,
                                  &value,
                                  &last_change])
                       .await?;
      }
    }
    Ok(())
  }
}
