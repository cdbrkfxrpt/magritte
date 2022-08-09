// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use serde::Deserialize;
use tokio::{task::JoinHandle, time};
use tracing::info;


#[derive(Debug, Deserialize)]
pub struct Broker {}

impl Broker {
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
