// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::node::{Node, NodeTx};
use crate::fluent::{Fluent, FluentTrait};

use eyre::Result;
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};
use tokio::{sync::{broadcast, mpsc},
            time};
use tokio_stream::StreamMap;
// use tracing::info;


#[derive(Debug, Deserialize)]
/// Core service of the application. Receives fluents from publisher nodes and
/// forwards them to subscriber nodes efficiently.
pub struct Broker {
  broadcast_capacity: usize,
  timeout:            u64,
  #[serde(skip)]
  fluents:            HashMap<String, broadcast::Sender<Fluent>>,
  #[serde(skip, default = "mpsc::unbounded_channel")]
  node_ch:            (NodeTx, mpsc::UnboundedReceiver<Fluent>),
}

impl Broker {
  /// Method to register a [`Node`] at the [`Broker`]. Creates channels to
  /// communicate fluents as required and initializes [`Node`]s accordingly.
  pub fn register<T: Node + ?Sized>(&mut self, node: &mut T) {
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
    let node_tx = self.node_ch.0.clone();
    node.initialize(node_tx, stream_map)
  }

  /// Runs the [`Broker`], receiving fluents from [`Node`]s and forwarding them
  /// to the  [`Node`]s which are subscribed to the respective fluent.
  pub async fn run(self) -> Result<()> {
    let mut input_rx = self.node_ch.1;
    let timeout_duration = Duration::from_secs(self.timeout);

    // TODO
    // better timeout message than the default "deadline has elapsed"
    while let Some(fluent) =
      time::timeout(timeout_duration, input_rx.recv()).await?
    {
      // info!("received fluent: {:?}", fluent);
      let fluent_name = fluent.name().to_string();
      // sending on a broadcast channel may return an error Result, which just
      // means that there are no receivers for this channel. however, it still
      // takes resources and since CURRENTLY all our receivers are known at
      // compile time, we can just skip that send entirely.
      if self.fluents[&fluent_name].receiver_count() == 0 {
        continue;
      }

      // now we can safely ? on the send; otherwise we could use a match here.
      self.fluents[&fluent_name].send(fluent)?;
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
