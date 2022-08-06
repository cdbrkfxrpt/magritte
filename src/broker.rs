// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use tokio::{task::JoinHandle, time};
use tracing::info;


#[derive(Debug)]
pub struct Broker {

}

impl Broker {
  pub fn init() -> Self {
    Self {}
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
