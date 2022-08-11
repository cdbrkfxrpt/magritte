// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Node, NodeTx};
use crate::fluent::AnyFluent;

use serde::Deserialize;
use std::collections::HashMap;
use tokio::{sync::{broadcast, mpsc},
            task::JoinHandle,
            time};
use tokio_stream::StreamMap;
use tracing::info;


type InputRx = mpsc::UnboundedReceiver<AnyFluent>;


#[derive(Debug, Deserialize)]
pub struct Broker {
  broadcast_capacity: usize,
  #[serde(skip)]
  fluents:            HashMap<String, broadcast::Sender<AnyFluent>>,
  #[serde(skip, default = "mpsc::unbounded_channel")]
  node_ch:            (NodeTx, InputRx),
}

impl Broker {
  pub fn register<T: Node>(&mut self, node: &mut T) {
    // add all fluents, whether published or subscribed, into the known list of
    // fluents, and create broadcast sender handles to them
    for fluent_name in node.published().iter().chain(node.subscribed().iter())
    {
      if !self.fluents.contains_key(fluent_name) {
        let (tx, _) = broadcast::channel(self.broadcast_capacity);
        self.fluents.insert(fluent_name.clone(), tx);
      }
    }

    // bundle all subscribed fluent receivers together in one stream
    let mut stream_map = StreamMap::new();
    for fluent_name in node.subscribed() {
      stream_map.insert(fluent_name.clone(),
                        self.fluents[&fluent_name].subscribe().into());
    }

    // initialize Node with NodeTx and NodeRx
    node.initialize(self.node_ch.0.clone(), stream_map)
  }

  pub fn run(self) -> JoinHandle<()> {
    tokio::spawn(async move {
      let mut counter = 0;
      let mut interval = time::interval(time::Duration::from_millis(256));
      loop {
        info!("counter value at {:8}", counter);
        interval.tick().await;
        counter += 1;
      }
    })
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
