// Copyright 2022 Florian Eich <florian.eich@gmail.com>
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_postgres::{row::Row, types::ToSql};


#[derive(Clone, Debug)]
pub struct Datapoint {
  pub source_id: usize,
  pub timestamp: usize,
  pub values:    HashMap<String, f64>,
}

impl Datapoint {
  pub fn new_datapoint(source_id: usize,
                       timestamp: usize,
                       value_names: &Vec<String>)
                       -> Self {
    let mut values = HashMap::new();
    for value_name in value_names {
      values.insert(value_name.to_owned(), 0.0f64);
    }

    Self { source_id,
           timestamp,
           values }
  }

  pub fn update_datapoint(&mut self, row: Row) {
    self.source_id = row.get::<&str, i32>("source_id") as usize;
    self.timestamp = row.get::<&str, i64>("timestamp") as usize;
    for (value_name, value) in self.values.iter_mut() {
      *value = row.get(value_name.as_str());
    }
  }
}


#[derive(Clone, Debug)]
pub struct FluentRequest {
  pub source_id:   Option<usize>,
  pub timestamp:   usize,
  pub name:        String,
  pub params:      Option<Vec<f64>>,
  pub response_tx: mpsc::Sender<FluentResult>,
}

impl FluentRequest {
  pub fn new(source_id: Option<usize>,
             timestamp: usize,
             name: &str,
             params: Option<Vec<f64>>,
             response_tx: mpsc::Sender<FluentResult>)
             -> Self {
    let name = name.to_owned();

    Self { source_id,
           timestamp,
           name,
           params,
           response_tx }
  }
}


#[derive(Clone, Debug)]
pub struct FluentResult {
  pub source_id: usize,
  pub timestamp: usize,
  pub name:      String,
  pub holds:     bool,
  pub params:    Option<Vec<f64>>,
}

impl FluentResult {
  pub fn new(source_id: usize,
             timestamp: usize,
             name: &str,
             holds: bool,
             params: Option<Vec<f64>>)
             -> Self {
    let name = name.to_owned();

    Self { source_id,
           timestamp,
           name,
           holds,
           params }
  }
}


#[derive(Debug)]
pub struct KnowledgeRequest {
  pub name:        String,
  pub source_id:   usize,
  pub timestamp:   usize,
  pub params:      Vec<Box<dyn ToSql + Sync + Send>>,
  pub response_tx: mpsc::Sender<Response>,
}

impl KnowledgeRequest {
  pub fn new(name: &str,
             source_id: usize,
             timestamp: usize,
             params: Vec<Box<dyn ToSql + Sync + Send>>,
             response_tx: mpsc::Sender<Response>)
             -> Self {
    let name = name.to_owned();

    Self { name,
           source_id,
           timestamp,
           params,
           response_tx }
  }
}


#[derive(Debug)]
pub struct Response {
  pub name:      String,
  pub source_id: usize,
  pub timestamp: usize,
  pub values:    Row,
}

impl Response {
  pub fn new(name: &str,
             source_id: usize,
             timestamp: usize,
             values: Row)
             -> Self {
    let name = name.to_owned();

    Self { name,
           source_id,
           timestamp,
           values }
  }
}


#[derive(Clone, Debug)]
pub enum BrokerMessage {
  Data(Datapoint),
  FluentReq(FluentRequest),
}


#[derive(Debug)]
pub enum BrokerRequest {
  KnowledgeReq(KnowledgeRequest),
  FluentReq(FluentRequest),
}
