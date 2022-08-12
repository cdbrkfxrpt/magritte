// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::fluent::AnyFluent;

use async_trait::async_trait;
use eyre::Result;
use tokio::sync::mpsc;
use tokio_postgres::Client as DatabaseClient;
use tokio_stream::{wrappers::BroadcastStream, StreamMap};


/// Helper type for [`Node`] senders. All [`Node`]s publish [`AnyFluent`]s to
/// the [`crate::services::Broker`] using this kind of sender.
pub type NodeTx = mpsc::UnboundedSender<AnyFluent>;

/// Helper type  for [`Node`] receivers. All [`Node`]s receive [`AnyFluent`]s
/// they are subscribed to using this kind of receiver.
pub type NodeRx = StreamMap<String, BroadcastStream<AnyFluent>>;


/// Any type implementing this trait can register itself as a node at the
/// [`Broker`] service.
pub trait Node: Send {
  /// Provides a list of [`AnyFluent`]s the node publishes.
  fn publishes(&self) -> Vec<String>;
  /// Provides a list of [`AnyFluent`]s the node subscribes to.
  fn subscribes_to(&self) -> Vec<String>;
  /// Initalizes the node with its [`NodeTx`] and [`NodeRx`] elements.
  fn initialize(&mut self, node_tx: NodeTx, node_rx: NodeRx);
}


/// Implemented by nodes which require database access.
#[async_trait]
pub trait StructuralNode: Node {
  async fn run(self, database_client: DatabaseClient) -> Result<()>;
}

/// Implemented by nodes  which evaluate [`AnyFluent`]s.
#[async_trait]
pub trait FluentNode: Node {
  async fn run(self) -> Result<()>;
}
