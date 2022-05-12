// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::Config, datapoint::DataPoint};

use eyre::{ensure, Result};
use std::collections::HashMap;
use tokio::{sync::{mpsc,
                   mpsc::{Receiver, Sender}},
            task::JoinHandle};
use tracing::info;


#[derive(Debug)]
pub struct Dispatcher {
  channel_capacity: usize,
  task:             Option<JoinHandle<()>>,
}

impl Dispatcher {
  pub fn new(config: &Config) -> Result<Self> {
    Ok(Self { channel_capacity: config.channel_capacity,
              task:             None, })
  }

  pub async fn start(&mut self,
                     mut linein: Receiver<DataPoint>)
                     -> Result<Receiver<DataPoint>> {
    ensure!(self.task.is_none(), "Dispatcher already running");

    let (cotx, lineout) = mpsc::channel(self.channel_capacity);

    let channel_capacity = self.channel_capacity;
    let task = tokio::spawn(async move {
      let mut sinks: HashMap<i32, Sender<DataPoint>> = HashMap::new();

      while let Some(dp) = linein.recv().await {
        info!("{}", dp);
        match sinks.get(dp.source()) {
          Some(tx) => {
            tx.send(dp).await.unwrap();
          }
          None => {
            let cotx = cotx.clone();
            let (tx, mut rx) = mpsc::channel(channel_capacity);
            sinks.insert(dp.source().clone(), tx.clone());

            tokio::spawn(async move {
              while let Some(dp) = rx.recv().await {
                info!("{}", dp);
                cotx.send(dp).await.unwrap();
              }
            });

            tx.send(dp).await.unwrap();
          }
        }
      }
    });
    self.task = Some(task);

    Ok(lineout)
  }
}


// TODO: write tests
//
