// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{fluent::AnyFluent,
            nodes::{Node, NodeTx}};

use eyre::Result;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::StreamMap;
// use tracing::info;


#[derive(Debug, Deserialize)]
/// Core service of the application. Receives fluents from publisher nodes and
/// forwards them to subscriber nodes efficiently.
pub struct Broker {
  broadcast_capacity: usize,
  #[serde(skip)]
  fluents:            HashMap<String, broadcast::Sender<AnyFluent>>,
  #[serde(skip, default = "mpsc::unbounded_channel")]
  node_ch:            (NodeTx, mpsc::UnboundedReceiver<AnyFluent>),
}

impl Broker {
  /// Method to register a [`Node`] at the [`Broker`]. Creates channels to
  /// communicate fluents as required and initializes [`Node`]s accordingly.
  pub fn register(&mut self, node: Box<&mut dyn Node>) {
    // add all fluents, whether published or subscribed to by the node, into
    // the known list of fluents, and create broadcast sender handles to them
    for fluent_name in
      node.publishes().iter().chain(node.subscribes_to().iter())
    {
      if !self.fluents.contains_key(fluent_name) {
        let (tx, _) = broadcast::channel(self.broadcast_capacity);
        self.fluents.insert(fluent_name.clone(), tx);
      }
    }

    // bundle all subscribed fluent receivers together in one stream
    let mut stream_map = StreamMap::new();
    for fluent_name in node.subscribes_to() {
      stream_map.insert(fluent_name.clone(),
                        self.fluents[&fluent_name].subscribe().into());
    }

    // initialize Node with NodeTx and NodeRx
    node.initialize(self.node_ch.0.clone(), stream_map)
  }

  /// Helper method to register a `Vec` of [`Node`]s at once. Iterates `Vec`,
  /// using the [`register`](Self::register) method to register them.
  pub fn register_all(&mut self, nodes: Vec<Box<&mut dyn Node>>) {
    for node in nodes {
      self.register(node);
    }
  }

  /// Runs the [`Broker`], receiving fluents from [`Node`]s and forwarding them
  /// to the  [`Node`]s which are subscribed to the respective fluent.
  pub async fn run(self) -> Result<()> {
    let mut input_rx = self.node_ch.1;
    while let Some(any_fluent) = input_rx.recv().await {
      self.fluents[any_fluent.name()].send(any_fluent)?;
    }
    Ok(())
  }
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  // use super::Broker;
  // use crate::fluent::Fluent;

  // use pretty_assertions::assert_eq;


  #[test]
  fn register_node_test() {
    // let mut broker = Broker::init();

    // let (tx, mut rx) = broker.register_node("i_am_fluent",
    //                                         ["subscription_one",
    //                                          "subscription_two"]);
  }
}
