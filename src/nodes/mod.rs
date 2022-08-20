// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

//! [`Node`]s connect to the [`Broker`](crate::app_core::Broker) to publish and
//! subscribe to [`Fluent`](crate::fluent::Fluent)s.
//!
//! Contains all the fun stuff required to implement nodes and send data
//! around between them.

mod sink;
mod source;

pub mod fluent_handler;
pub use sink::Sink;
pub use source::Source;

// fin re-exports ---------------------------------------------------------- //

use crate::fluent::Fluent;

use std::fmt;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::BroadcastStream, StreamMap};


/// Helper type for [`Node`] senders. All [`Node`]s publish [`Fluent`]s to
/// the [`Broker`](crate::app_core::Broker) using this kind of sender.
pub type NodeTx = mpsc::UnboundedSender<Fluent>;

/// Helper type  for [`Node`] receivers. All [`Node`]s receive [`Fluent`]s
/// they are subscribed to using this kind of receiver.
pub type NodeRx = StreamMap<String, BroadcastStream<Fluent>>;


/// Any type implementing this trait can register itself as a node at the
/// [`Broker`](crate::app_core::Broker) service.
pub trait Node: fmt::Debug + Send {
  /// Provides a list of [`Fluent`]s the node publishes.
  fn publishes(&self) -> Vec<String>;
  /// Provides a list of [`Fluent`]s the node subscribes to.
  fn subscribes_to(&self) -> Vec<String>;
  /// Initalizes the node with its [`NodeTx`] and [`NodeRx`] elements.
  fn initialize(&mut self, node_tx: NodeTx, node_rx: NodeRx);
}
