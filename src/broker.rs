// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::Config,
            knowledge::build_functions_index,
            source::Source,
            types::{BrokerMessage,
                    BrokerRequest,
                    Datapoint,
                    FluentResult,
                    Response}};

use std::collections::HashMap;
use tokio::{sync::{mpsc, mpsc::error::TryRecvError},
            task::JoinHandle,
            time};
use tokio_postgres::{types::ToSql, NoTls};
use tracing::{error, info};


#[derive(Debug)]
pub struct Broker {
  config:  Config,
  data_rx: mpsc::Receiver<Datapoint>,
  sink_tx: mpsc::Sender<FluentResult>,
  sources: HashMap<usize, mpsc::Sender<BrokerMessage>>,
}

impl Broker {
  pub fn init(
    config: &Config)
    -> (Self, mpsc::Sender<Datapoint>, mpsc::Receiver<FluentResult>) {
    let config = config.clone();
    let (data_tx, data_rx) = mpsc::channel(32);
    let (sink_tx, sink_rx) = mpsc::channel(32);
    let sources = HashMap::new();

    (Self { config,
            data_rx,
            sink_tx,
            sources },
     data_tx,
     sink_rx)
  }

  pub fn run(mut self) -> JoinHandle<()> {
    tokio::spawn(async move {
      let dbparams = format!("host={} user={} password={} dbname={}",
                             self.config.database_credentials.host,
                             self.config.database_credentials.user,
                             self.config.database_credentials.password,
                             self.config.database_credentials.dbname);

      let (dbclient, connection) =
        tokio_postgres::connect(&dbparams, NoTls).await.unwrap();

      info!("database connection successful");

      // task awaits database connection, traces on error
      tokio::spawn(async move {
        if let Err(e) = connection.await {
          error!("connection error: {}", e);
        }
      });

      let function_statements = build_functions_index(&dbclient).await;
      let (request_tx, mut request_rx) = mpsc::channel(32);

      let mut interval = time::interval(time::Duration::from_millis(1));

      loop {
        interval.tick().await;

        // handle requests
        loop {
          match request_rx.try_recv() {
            Ok(request) => match request {
              BrokerRequest::FluentReq(fluent_request) => {
                let source = fluent_request.source_id.clone();
                let broker_message = BrokerMessage::FluentReq(fluent_request);

                if let Some(source_id) = source {
                  self.sources[&source_id].send(broker_message).await.unwrap();
                } else {
                  for (source_id, source_tx) in self.sources.iter() {
                    if source_id == &228051000 {
                      info!("source_tx capacity: {}", source_tx.capacity());
                    }
                    source_tx.send(broker_message.clone()).await.unwrap();
                  }
                }
              }
              BrokerRequest::KnowledgeReq(knowledge_request) => {
                let statement =
                  function_statements.get(&knowledge_request.name).unwrap();
                let params =
                  knowledge_request.params
                                   .iter()
                                   .map(|e| e.as_ref() as &(dyn ToSql + Sync))
                                   .collect::<Vec<_>>();

                let rows =
                  dbclient.query(statement, params.as_slice()).await.unwrap();
                for row in rows {
                  let response = Response::new(&knowledge_request.name,
                                               knowledge_request.source_id,
                                               knowledge_request.timestamp,
                                               row);

                  knowledge_request.response_tx.send(response).await.unwrap();
                }
              }
            },
            Err(TryRecvError::Empty) => {
              // info!("request_rx empty, breaking from loop");
              break;
            }
            Err(TryRecvError::Disconnected) => return,
          }
        }

        // handle new datapoints
        loop {
          match self.data_rx.try_recv() {
            Ok(datapoint) => {
              let source_tx =
                if !self.sources.contains_key(&datapoint.source_id) {
                  let (source_tx, source_rx) = mpsc::channel(32);

                  self.sources.insert(datapoint.source_id, source_tx.clone());
                  Source::init(datapoint.source_id,
                               source_rx,
                               request_tx.clone(),
                               self.sink_tx.clone()).run();

                  source_tx
                } else {
                  self.sources[&datapoint.source_id].clone()
                };

              if datapoint.source_id == 228051000 {
                info!("source_tx capacity: {}", source_tx.capacity());
              }
              source_tx.send(BrokerMessage::Data(datapoint))
                       .await
                       .unwrap();
            }
            Err(TryRecvError::Empty) => {
              // info!("data_rx empty, breaking from loop");
              break;
            }
            Err(TryRecvError::Disconnected) => return,
          }
        }
      }
    })
  }
}
