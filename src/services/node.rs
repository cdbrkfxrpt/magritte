// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::fluent::AnyFluent;

use tokio::sync::mpsc;
use tokio_stream::{wrappers::BroadcastStream, StreamMap};


pub type NodeTx = mpsc::UnboundedSender<AnyFluent>;
pub type NodeRx = StreamMap<String, BroadcastStream<AnyFluent>>;


pub trait Node: Send {
  fn publishes(&self) -> Vec<String>;
  fn subscribes_to(&self) -> Vec<String>;
  fn initialize(&mut self, node_tx: NodeTx, node_rx: NodeRx);
}
