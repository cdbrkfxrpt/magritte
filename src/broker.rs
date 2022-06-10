// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::{Config, DatabaseConnection},
            knowledge::build_functions_index,
            source::Source,
            types::{BrokerMessage,
                    BrokerRequest,
                    Datapoint,
                    FluentResult,
                    Response}};

use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tokio_postgres::{types::ToSql, NoTls};
use tracing::{error, info};


type Sources = Arc<Mutex<HashMap<usize, mpsc::Sender<BrokerMessage>>>>;


#[derive(Debug)]
pub struct Broker {
  config:          Config,
  request_tx:      mpsc::Sender<BrokerRequest>,
  sources:         Sources,
  data_handler:    DataHandler,
  request_handler: RequestHandler,
}

impl Broker {
  pub fn init(config: Config,
              feeder_rx: mpsc::Receiver<Datapoint>,
              sink_tx: mpsc::Sender<FluentResult>)
              -> Self {
    let (request_tx, request_rx) =
      mpsc::channel(config.broker.channel_capacity);
    let sources = Arc::new(Mutex::new(HashMap::new()));

    let data_handler = DataHandler::init(feeder_rx,
                                         sink_tx,
                                         request_tx.clone(),
                                         sources.clone());

    let request_handler =
      RequestHandler::init(&config.database, request_rx, sources.clone());

    Self { config,
           request_tx,
           sources,
           data_handler,
           request_handler }
  }

  pub fn run(self) {
    self.data_handler.run(self.config.broker.channel_capacity);
    self.request_handler.run();
  }
}


#[derive(Debug)]
pub struct DataHandler {
  feeder_rx:  mpsc::Receiver<Datapoint>,
  sink_tx:    mpsc::Sender<FluentResult>,
  request_tx: mpsc::Sender<BrokerRequest>,
  sources:    Sources,
}

impl DataHandler {
  pub fn init(feeder_rx: mpsc::Receiver<Datapoint>,
              sink_tx: mpsc::Sender<FluentResult>,
              request_tx: mpsc::Sender<BrokerRequest>,
              sources: Sources)
              -> Self {
    Self { feeder_rx,
           sink_tx,
           request_tx,
           sources }
  }

  pub fn run(mut self, channel_capacity: usize) {
    tokio::spawn(async move {
      // source spawning and data sending task
      while let Some(datapoint) = self.feeder_rx.recv().await {
        let source = datapoint.source_id;

        // unwrapping Mutex lock is safe per Mutex docs
        let mut sources = self.sources.lock().await;

        let source_tx = if !sources.contains_key(&datapoint.source_id) {
          let (source_tx, source_rx) = mpsc::channel(channel_capacity);

          sources.insert(datapoint.source_id, source_tx.clone());
          Source::init(datapoint.source_id,
                       source_rx,
                       self.request_tx.clone(),
                       self.sink_tx.clone()).run();

          source_tx
        } else {
          sources[&datapoint.source_id].clone()
        };

        info!("found or inserted source {}", source);

        source_tx.send(BrokerMessage::Data(datapoint))
                 .await
                 .unwrap();

        info!("sent datapoint to source {}", source);
      }
    });
  }
}


#[derive(Debug)]
pub struct RequestHandler {
  config:     DatabaseConnection,
  request_rx: mpsc::Receiver<BrokerRequest>,
  sources:    Sources,
}

impl RequestHandler {
  pub fn init(config: &DatabaseConnection,
              request_rx: mpsc::Receiver<BrokerRequest>,
              sources: Sources)
              -> Self {
    let config = config.clone();
    Self { config,
           request_rx,
           sources }
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      let dbparams = format!("host={} user={} password={} dbname={}",
                             self.config.host,
                             self.config.user,
                             self.config.password,
                             self.config.dbname);

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

      while let Some(request) = self.request_rx.recv().await {
        match request {
          BrokerRequest::FluentReq(fluent_request) => {
            let sources = self.sources.lock().await;
            let outmsg = BrokerMessage::FluentReq(fluent_request.clone());

            if let Some(source_id) = fluent_request.source_id {
              sources[&source_id].clone().send(outmsg).await.unwrap();
            } else {
              for (_, source_tx) in sources.iter() {
                source_tx.send(outmsg.clone()).await.unwrap();
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
        }
      }
    });
  }
}
