// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Node, NodeRx, NodeTx};
use crate::fluent::AnyFluent;

use async_trait::async_trait;
use eyre::{bail, eyre, Result};
use serde::Deserialize;
use tokio_postgres::Client;
use tokio_stream::StreamExt;
use tracing::info;


#[derive(Debug, Deserialize)]
/// Receives [`AnyFluent`]s from the [`Broker`](crate::app_core::Broker)
/// service and writes them to the PostgreSQL database.
pub struct Sink {
  subscribes_to: Vec<String>,
  #[serde(skip)]
  node_rx:       Option<NodeRx>,
}

#[async_trait]
impl Node for Sink {
  /// `Sink` publishes no fluents. Implementation returns empty `Vec`.
  fn publishes(&self) -> Vec<String> {
    Vec::new()
  }

  /// `Sink` subscribes to fluents specified by name in the app configuration.
  fn subscribes_to(&self) -> Vec<String> {
    self.subscribes_to.clone()
  }

  /// `Sink` requires a database client, so this method returns `true`.
  fn requires_dbc(&self) -> bool {
    true
  }

  /// `Sink` requires only a receiver handle since it publishes no fluents.
  fn initialize(&mut self, _: NodeTx, node_rx: NodeRx) {
    self.node_rx = Some(node_rx);
  }

  /// Runs the [`Sink`], receiving fluents from the
  /// [`Broker`](crate::app_core::Broker) and writing them to the database.
  /// Consumes the original object.
  async fn run(mut self: Box<Self>,
               database_client: Option<Client>)
               -> Result<()> {
    let database_client =
      database_client.ok_or(eyre!("Sink requires a database client"))?;

    let mut node_rx = match self.node_rx {
      Some(node_rx) => node_rx,
      None => bail!("Sink not initialized, aborting"),
    };

    let sql_raw = include_str!("../sql/sink.sql");

    while let Some((_, Ok(any_fluent))) = node_rx.next().await {
      info!("Sink received: {:?}", any_fluent);
      let AnyFluent::Boolean(fluent) = any_fluent else {
        continue;
        // bail!("Sink received non-boolean fluent, aborting")
      };

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
    Ok(())
  }
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::{Node, Sink};
  use crate::stringvec;

  use pretty_assertions::assert_eq;

  fn sink_init() -> Sink {
    Sink { subscribes_to: stringvec!["highSpeedNearCoast", "rendezVous"],
           node_rx:       None, }
  }

  #[test]
  fn sink_test() {
    let sink = sink_init();

    assert_eq!(sink.publishes(), Vec::<String>::new());
    assert_eq!(sink.subscribes_to,
               stringvec!["highSpeedNearCoast", "rendezVous"]);
    assert_eq!(sink.subscribes_to(),
               stringvec!["highSpeedNearCoast", "rendezVous"]);
    assert!(sink.node_rx.is_none());
  }
}
