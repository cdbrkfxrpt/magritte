// Copyright 2022 Florian Eich <florian.eich@gmail.com>
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_postgres::row::Row;
use tracing::error;


#[derive(Clone, Debug)]
pub enum Message {
  Datapoint {
    source_id: usize,
    timestamp: usize,
    values:    HashMap<String, f64>,
  },
  Request {
    request_type: RequestType,
    fn_name:      String,
    source_id:    Option<usize>,
    timestamp:    usize,
    params:       HashMap<String, f64>,
    response_tx:  mpsc::Sender<Message>,
  },
  Response {
    fn_name:     String,
    source_id:   Option<usize>,
    timestamp:   usize,
    values:      HashMap<String, f64>,
    rule_result: RuleResult,
  },
}


impl Message {
  pub fn new_datapoint(source_id: usize,
                       timestamp: usize,
                       value_names: &Vec<String>)
                       -> Self {
    let mut values = HashMap::new();
    for value_name in value_names {
      values.insert(value_name.to_owned(), 0.0f64);
    }

    Self::Datapoint { source_id,
                      timestamp,
                      values }
  }

  pub fn update_datapoint(&mut self, row: Row) {
    match self {
      Message::Datapoint { source_id,
                           timestamp,
                           values, } => {
        *source_id = row.get::<&str, i32>("source_id") as usize;
        *timestamp = row.get::<&str, i64>("timestamp") as usize;
        for (value_name, value) in values.iter_mut() {
          *value = row.get(value_name.as_str());
        }
      }
      _ => error!("unable to call update_datapoint on non-Datapoint"),
    }
  }

  pub fn to_response(self, fn_name: String, rule_result: RuleResult) -> Self {
    let (source_id, timestamp, values) = match self {
      Message::Datapoint { source_id,
                           timestamp,
                           values, } => (Some(source_id), timestamp, values),
      Message::Request { source_id,
                         timestamp,
                         params,
                         .. } => (source_id, timestamp, params),
      Message::Response { source_id,
                          timestamp,
                          values,
                          .. } => (source_id, timestamp, values),
    };

    Self::Response { fn_name,
                     source_id,
                     timestamp,
                     values,
                     rule_result }
  }
}


#[derive(Clone, Debug)]
pub enum RequestType {
  SourceRequest,
  KnowledgeRequest,
}


#[derive(Clone, Debug)]
pub enum RuleResult {
  Boolean(bool),
  Numeric(f64),
}
