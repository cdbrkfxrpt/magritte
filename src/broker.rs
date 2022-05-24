// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{config::{BrokerConfig, DatabaseConnection},
            knowledge::build_functions_index,
            source::Source,
            types::{Message, RequestType, RuleResult}};

use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tokio_postgres::{types::ToSql, NoTls};
use tracing::{error, info};


type Sources = Arc<Mutex<HashMap<usize, mpsc::Sender<Message>>>>;


#[derive(Debug)]
pub struct Broker {
  config:          BrokerConfig,
  request_tx:      mpsc::Sender<Message>,
  sources:         Sources,
  data_handler:    DataHandler,
  request_handler: RequestHandler,
}

impl Broker {
  pub fn init(config: BrokerConfig,
              feeder_rx: mpsc::Receiver<Message>,
              sink_tx: mpsc::Sender<Message>)
              -> Self {
    let (request_tx, request_rx) = mpsc::channel(config.channel_capacity);
    let sources = Arc::new(Mutex::new(HashMap::new()));

    let data_handler = DataHandler::init(feeder_rx,
                                         sink_tx,
                                         request_tx.clone(),
                                         sources.clone());

    let request_handler =
      RequestHandler::init(&config.connection, request_rx, sources.clone());

    Self { config,
           request_tx,
           sources,
           data_handler,
           request_handler }
  }

  pub fn run(self) {
    self.data_handler.run(self.config.channel_capacity);
    self.request_handler.run();
  }
}


#[derive(Debug)]
pub struct DataHandler {
  feeder_rx:  mpsc::Receiver<Message>,
  sink_tx:    mpsc::Sender<Message>,
  request_tx: mpsc::Sender<Message>,
  sources:    Sources,
}

impl DataHandler {
  pub fn init(feeder_rx: mpsc::Receiver<Message>,
              sink_tx: mpsc::Sender<Message>,
              request_tx: mpsc::Sender<Message>,
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
      while let Some(message) = self.feeder_rx.recv().await {
        // unwrapping Mutex lock is safe per Mutex docs
        let mut sources = self.sources.lock().await;

        let Message::Datapoint { source_id, .. } = message else {
          panic!("message sent from Feeder not a datapoint");
        };

        let source_tx = if !sources.contains_key(&source_id) {
          let (source_tx, source_rx) = mpsc::channel(channel_capacity);

          sources.insert(source_id, source_tx.clone());
          Source::init(source_id,
                       source_rx,
                       self.request_tx.clone(),
                       self.sink_tx.clone()).run();

          source_tx
        } else {
          sources[&source_id].clone()
        };

        source_tx.send(message).await.unwrap();
      }
    });
  }
}


#[derive(Debug)]
pub struct RequestHandler {
  config:     DatabaseConnection,
  request_rx: mpsc::Receiver<Message>,
  sources:    Sources,
}

impl RequestHandler {
  pub fn init(config: &DatabaseConnection,
              request_rx: mpsc::Receiver<Message>,
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

      while let Some(message) = self.request_rx.recv().await {
        let Message::Request { request_type,
                               fn_name,
                               source_id,
                               timestamp,
                               params,
                               response_tx } = message.clone() else {
            panic!("message received by RequestHandler not a request");
          };

        match request_type {
          RequestType::SourceRequest => {
            let sources = self.sources.lock().await;
            if let Some(source_id) = source_id {
              sources[&source_id].clone().send(message).await.unwrap();
            } else {
              for (_, source_tx) in sources.iter() {
                source_tx.send(message.clone()).await.unwrap();
              }
            }
          }
          RequestType::KnowledgeRequest => {
            let statement = function_statements.get(&fn_name).unwrap();

            let rows = dbclient.query(statement,
                                      params.iter()
                                            .map(|e| e as &(dyn ToSql + Sync))
                                            .collect::<Vec<_>>()
                                            .as_slice())
                               .await
                               .unwrap();

            for row in rows {
              let mut values: HashMap<String, f64> = HashMap::new();
              for col in row.columns() {
                values.insert(col.name().to_owned(), row.get(col.name()));
              }
              let response = Message::new_response(&fn_name,
                                                   // unwrap is safe,
                                                   // knowledgerequests
                                                   // always have a source_id
                                                   source_id.unwrap(),
                                                   timestamp,
                                                   values,
                                                   RuleResult::Boolean(true));

              response_tx.send(response).await.unwrap();
            }
          }
        }
      }
    });
  }
}
