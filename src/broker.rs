// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::{Config, DatabaseCredentials},
            knowledge::build_functions_index,
            source::Source,
            types::{BrokerMessage,
                    BrokerRequest,
                    Datapoint,
                    FluentResult,
                    Response}};

use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_postgres::{types::ToSql, NoTls};
use tracing::{error, info};


type InnerSender = mpsc::Sender<(Option<usize>, BrokerMessage)>;
type InnerReceiver = mpsc::Receiver<(Option<usize>, BrokerMessage)>;


#[derive(Debug)]
pub struct Broker {
  inner_rx:        InnerReceiver,
  sink_tx:         mpsc::Sender<FluentResult>,
  data_handler:    DataHandler,
  request_tx:      mpsc::Sender<BrokerRequest>,
  request_handler: RequestHandler,
  source_capacity: usize,
  sources:         HashMap<usize, mpsc::Sender<BrokerMessage>>,
}

impl Broker {
  pub fn init(
    config: &Config)
    -> (Self, mpsc::Sender<Datapoint>, mpsc::Receiver<FluentResult>) {
    let (inner_tx, inner_rx) = mpsc::channel(config.channel_capacities.inner);
    let (sink_tx, sink_rx) = mpsc::channel(config.channel_capacities.sink);

    let (request_handler, request_tx) =
      RequestHandler::init(inner_tx.clone(),
                           config.channel_capacities.request,
                           &config.database_credentials);

    let (data_handler, data_tx) =
      DataHandler::init(inner_tx.clone(), config.channel_capacities.data);

    (Self { inner_rx,
            sink_tx,
            data_handler,
            request_tx,
            request_handler,
            source_capacity: config.channel_capacities.source,
            sources: HashMap::new() },
     data_tx,
     sink_rx)
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      self.request_handler.run();
      self.data_handler.run();

      while let Some((receiver, broker_message)) = self.inner_rx.recv().await {
        if let Some(source_id) = receiver {
          let source_tx = if !self.sources.contains_key(&source_id) {
            let (source_tx, source_rx) = mpsc::channel(self.source_capacity);

            self.sources.insert(source_id, source_tx.clone());
            Source::init(source_id,
                         source_rx,
                         self.request_tx.clone(),
                         self.sink_tx.clone()).run();

            source_tx
          } else {
            self.sources[&source_id].clone()
          };

          source_tx.send(broker_message).await.unwrap();
        } else {
          for (_, source_tx) in self.sources.iter() {
            source_tx.send(broker_message.clone()).await.unwrap();
          }
        }
      }
    });
  }
}


#[derive(Debug)]
pub struct RequestHandler {
  inner_tx:             InnerSender,
  database_credentials: DatabaseCredentials,
  request_rx:           mpsc::Receiver<BrokerRequest>,
}

impl RequestHandler {
  pub fn init(inner_tx: InnerSender,
              channel_capacity: usize,
              database_credentials: &DatabaseCredentials)
              -> (Self, mpsc::Sender<BrokerRequest>) {
    let database_credentials = database_credentials.clone();
    let (request_tx, request_rx) = mpsc::channel(channel_capacity);

    (Self { inner_tx,
            database_credentials,
            request_rx },
     request_tx)
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      let dbparams = format!("host={} user={} password={} dbname={}",
                             self.database_credentials.host,
                             self.database_credentials.user,
                             self.database_credentials.password,
                             self.database_credentials.dbname);

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
            self.inner_tx
                .send((fluent_request.source_id,
                       BrokerMessage::FluentReq(fluent_request)))
                .await
                .unwrap();
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


#[derive(Debug)]
pub struct DataHandler {
  inner_tx: InnerSender,
  data_rx:  mpsc::Receiver<Datapoint>,
}

impl DataHandler {
  pub fn init(inner_tx: InnerSender,
              channel_capacity: usize)
              -> (Self, mpsc::Sender<Datapoint>) {
    let (data_tx, data_rx) = mpsc::channel(channel_capacity);

    (Self { inner_tx, data_rx }, data_tx)
  }

  pub fn run(mut self) {
    tokio::spawn(async move {
      // source spawning and data sending task
      while let Some(datapoint) = self.data_rx.recv().await {
        self.inner_tx
            .send((Some(datapoint.source_id), BrokerMessage::Data(datapoint)))
            .await
            .unwrap();
      }
    });
  }
}
