// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Node, NodeRx, NodeTx};
use crate::{fluent::{Fluent, FluentTrait},
            sqlvec};

use eyre::{bail, eyre, Result};
use serde::Deserialize;
use std::time::Duration;
use tokio::time;
use tokio_postgres::{types::ToSql, Client};
use tokio_stream::StreamExt;
use tracing::{debug, info};


#[derive(Debug, Deserialize)]
/// Receives [`Fluent`]s from the [`Broker`](super::broker::Broker)
/// service and writes them to the PostgreSQL database.
pub struct Sink {
  debug:         bool,
  write_timeout: usize,
  subscribes_to: Vec<String>,
  #[serde(skip)]
  node_rx:       Option<NodeRx>,
}

impl Sink {
  /// Runs the [`Sink`], receiving fluents from the
  /// [`Broker`](super::broker::Broker) and writing them to the database.
  /// Consumes the original object.
  pub async fn run(self, database_client: Option<Client>) -> Result<()> {
    let database_client =
      database_client.ok_or(eyre!("Sink requires a database client"))?;

    let mut node_rx = match self.node_rx {
      Some(node_rx) => node_rx,
      None => bail!("Sink not initialized, aborting"),
    };

    let statement_raw = include_str!("./sql/sink.sql");
    let statement = match database_client.prepare(statement_raw).await {
      Ok(statement) => statement,
      Err(err) => {
        panic!("Error in Sink PostgreSQL statement: {}", err);
      }
    };
    let timeout = Duration::from_millis(self.write_timeout as u64);

    while let Some((_, Ok(fluent))) = node_rx.next().await {
      // write only Boolean fluents to database
      if self.debug || !matches!(fluent, Fluent::Boolean(_)) {
        // info!("{}: {:?}", fluent.name(), fluent.boxed_value());
        continue;
      }

      let name = fluent.name();
      let keys = fluent.keys().iter().map(|&e| e as i32).collect::<Vec<_>>();
      let timestamp = fluent.timestamp() as i64;
      let value = fluent.value::<bool>();
      let last_change = fluent.last_change() as i64;

      let args = sqlvec![&name, &keys, &timestamp, &value, &last_change];
      let write_future = database_client.execute(&statement, args.as_slice());

      match time::timeout(timeout, write_future).await {
        Ok(result) => match result {
          Ok(rows_affected) => debug!("{} rows affected", rows_affected),
          Err(_) => info!("unable to write to database"),
        },
        Err(_) => info!("database write timed out"),
      }
    }
    Ok(())
  }
}

impl Node for Sink {
  /// `Sink` publishes no fluents. Implementation returns empty `Vec`.
  fn publishes(&self) -> Vec<String> {
    Vec::new()
  }

  /// `Sink` subscribes to fluents specified by name in the app configuration.
  fn subscribes_to(&self) -> Vec<String> {
    self.subscribes_to.clone()
  }

  /// `Sink` requires only a receiver handle since it publishes no fluents.
  fn initialize(&mut self, _: NodeTx, node_rx: NodeRx) {
    self.node_rx = Some(node_rx);
  }
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::{Node, Sink};
  use crate::stringvec;

  use pretty_assertions::assert_eq;

  fn sink_init() -> Sink {
    Sink { debug:         true,
           write_timeout: 42,
           subscribes_to: stringvec!["highSpeedNearCoast", "rendezVous"],
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
