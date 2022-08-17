// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{app_core::util::unordered_congruent,
            fluent::{AnyFluent, FluentValue, Timestamp},
            nodes::{KnowledgeNode, Node, NodeRx, NodeTx}};

use async_trait::async_trait;
use boolinator::Boolinator;
use eyre::{bail, Result};
use std::collections::BTreeMap;
use tokio_postgres::{types::ToSql, Client};
use tokio_stream::StreamExt;


#[derive(Debug)]
pub enum ResponseValueType {
  Textual,
  Integer,
  FloatPt,
  Boolean,
}


#[derive(Debug)]
/// A [`Node`] which handles the progression of fluents by subscribing to
/// dependency fluents and using incoming values to query the database for
/// background knowledge, producing an output fluent which is published as an
/// [`AnyFluent`] to the [`Broker`](crate::app_core::Broker).
pub struct KnowledgeHandler {
  name:         String,
  dependencies: Vec<String>,
  query_cmd:    String,
  rvt:          ResponseValueType,
  deps_buffer:  BTreeMap<Timestamp, Vec<AnyFluent>>,
  // history:                  Vec<AnyFluent>,
  // window:                   Option<usize>,
  node_ch:      Option<(NodeTx, NodeRx)>,
}

impl KnowledgeHandler {
  /// Instantiate a [`KnowledgeHandler`] with the name and dependencies of the
  /// [`AnyFluent`] it handles, as well as an evaluation function of type
  /// [`EvalFn`] (which is a wrapper struct for a closure).
  pub fn new(name: &str,
             dependencies: &[&str],
             rvt: ResponseValueType,
             query_cmd: &str)
             -> Self {
    let name = name.to_owned();
    let dependencies = dependencies.iter()
                                   .map(|e| e.to_string())
                                   .collect::<Vec<_>>();
    let query_cmd = query_cmd.to_owned();
    let deps_buffer = BTreeMap::new();
    let node_ch = None;

    Self { name,
           dependencies,
           query_cmd,
           rvt,
           deps_buffer,
           node_ch }
  }
}

impl Node for KnowledgeHandler {
  fn publishes(&self) -> Vec<String> {
    vec![self.name.clone()]
  }

  fn subscribes_to(&self) -> Vec<String> {
    self.dependencies.clone()
  }

  fn initialize(&mut self, node_tx: NodeTx, node_rx: NodeRx) {
    self.node_ch = Some((node_tx, node_rx));
  }
}

#[async_trait]
impl KnowledgeNode for KnowledgeHandler {
  /// This function contains a lot of the logic which defines the way fluents
  /// evolve over time, i.e. it contains the way dependencies are buffered,
  /// stored and detected to be complete for function evaluation, how
  /// background knowledge is obtained and provided to the evaluation function,
  /// how and when fluents are updated and published, all of that.
  async fn run(mut self: Box<Self>, database_client: Client) -> Result<()> {
    let (node_tx, mut node_rx) = match self.node_ch {
      Some((node_tx, node_rx)) => (node_tx, node_rx),
      None => {
        bail!("KnowledgeHandler '{}' not initialized, aborting", self.name)
      }
    };

    let statement = match database_client.prepare(&self.query_cmd).await {
      Ok(statement) => statement,
      Err(err) => {
        // drop(self.fluent_tx);
        panic!("Error in Feeder PostgreSQL query: {}", err);
      }
    };

    while let Some((_fluent_name, Ok(any_fluent))) = node_rx.next().await {
      // info!("KnowledgeHandler '{}' received: {:?}", self.name, any_fluent);

      let keys = any_fluent.keys().to_vec();
      let timestamp = any_fluent.timestamp();

      match self.deps_buffer.get_mut(&timestamp) {
        Some(buffer) => buffer.push(any_fluent),
        None => {
          self.deps_buffer.insert(timestamp, vec![any_fluent]);
        }
      }

      // deps: dependecies filtered by key as Vec
      let mut deps =
        self.deps_buffer[&timestamp].iter()
                                    .filter_map(|f| {
                                      (f.keys() == keys).as_some(f.clone())
                                    })
                                    .collect::<Vec<AnyFluent>>();

      let dep_names = deps.iter()
                          .map(|k| k.name().to_string())
                          .collect::<Vec<_>>();

      if self.dependencies.len() != dep_names.len()
         || !unordered_congruent(&self.dependencies, &dep_names)
      {
        continue;
      }

      // now that we've checked that deps contains all elements required by
      // self.dependencies, we can safely unwrap the position calls here:
      deps.sort_by(|lhs, rhs| {
            let lhs_pos = self.dependencies
                              .iter()
                              .position(|name| name == lhs.name())
                              .unwrap();
            let rhs_pos = self.dependencies
                              .iter()
                              .position(|name| name == rhs.name())
                              .unwrap();
            lhs_pos.cmp(&rhs_pos)
          });

      let values = deps.into_iter()
                       .map(|af| Into::<Vec<&(dyn ToSql + Sync)>>::into(af))
                       .collect::<Vec<_>>()
                       .into_iter()
                       .flatten()
                       .collect::<Vec<_>>();

      let mut rows =
        database_client.query(&statement, values.as_slice()).await?;
      let Some(row) = rows.pop() else { continue; };

      let response: Box<dyn FluentValue> = match self.rvt {
        ResponseValueType::Textual => Box::new(row.get::<usize, String>(0)),
        ResponseValueType::Integer => Box::new(row.get::<usize, i64>(0)),
        ResponseValueType::FloatPt => Box::new(row.get::<usize, f64>(0)),
        ResponseValueType::Boolean => Box::new(row.get::<usize, bool>(0)),
      };

      let fluent = AnyFluent::new(&self.name, &keys, timestamp, response);

      node_tx.send(fluent)?;
    }

    Ok(())
  }
}
