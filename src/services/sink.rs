// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Node, NodeRx, NodeTx};
use crate::fluent::AnyFluent;

use eyre::{bail, Result};
use serde::Deserialize;
use tokio_postgres::Client;
use tokio_stream::StreamExt;
// use tracing::info;


#[derive(Debug, Deserialize)]
pub struct Sink {
  subscribes_to: Vec<String>,
  #[serde(skip)]
  node_rx:       Option<NodeRx>,
}

impl Sink {
  pub async fn run(self, database_client: Client) -> Result<()> {
    let mut node_rx = match self.node_rx {
      Some(node_rx) => node_rx,
      None => bail!("Sink not initialized, aborting"),
    };

    let sql_raw = include_str!("sink.sql");

    while let Some((_, Ok(fluent))) = node_rx.next().await {
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

impl Node for Sink {
  fn publishes(&self) -> Vec<String> {
    Vec::new()
  }

  fn subscribes_to(&self) -> Vec<String> {
    self.subscribes_to.clone()
  }

  fn initialize(&mut self, _: NodeTx, node_rx: NodeRx) {
    self.node_rx = Some(node_rx);
  }
}
