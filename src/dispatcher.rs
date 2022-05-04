// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::datapoint::DataPoint;

use eyre::Result;
use std::collections::HashMap;
use tokio::{sync::{mpsc,
                   mpsc::{Receiver, Sender}},
            task::JoinHandle};
use tracing::info;


pub async fn start(mut linein: Receiver<DataPoint>)
                   -> Result<(JoinHandle<()>, Receiver<DataPoint>)> {
  let (cotx, lineout) = mpsc::channel(32);

  let handle = tokio::spawn(async move {
    let mut sinks: HashMap<i32, Sender<DataPoint>> = HashMap::new();

    while let Some(dp) = linein.recv().await {
      info!("{}", dp);
      match sinks.get(dp.source()) {
        Some(tx) => {
          tx.send(dp).await.unwrap();
        }
        None => {
          let cotx = cotx.clone();
          let (tx, mut rx) = mpsc::channel(32);
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

  Ok((handle, lineout))
}


// TODO: write tests
//
