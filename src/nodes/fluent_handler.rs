// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{app_core::RequestTx,
            fluent::{AnyFluent, FluentValue, Timestamp},
            nodes::{FluentNode, Node, NodeRx, NodeTx}};

use async_trait::async_trait;
use eyre::{bail, Result};
use std::collections::BTreeMap;


#[derive(Debug)]
pub struct FluentHandler<T, F>
  where T: FluentValue,
        F: Fn(Vec<AnyFluent>) -> T + Send {
  name:         String,
  dependencies: Vec<String>,
  eval_fn:      F,
  deps_buffer:  BTreeMap<Timestamp, Vec<AnyFluent>>,
  // history:                  Vec<AnyFluent>,
  // window:                   Option<usize>,
  node_ch:      Option<(NodeTx, NodeRx)>,
}

impl<T, F> FluentHandler<T, F>
  where T: FluentValue,
        F: Fn(Vec<AnyFluent>) -> T + Send
{
  pub fn new(name: &str, dependencies: &[&str], eval_fn: F) -> Self {
    let name = name.to_owned();
    let dependencies = dependencies.iter()
                                   .map(|e| e.to_string())
                                   .collect::<Vec<_>>();
    let deps_buffer = BTreeMap::new();
    let node_ch = None;

    Self { name,
           dependencies,
           eval_fn,
           deps_buffer,
           node_ch }
  }
}

impl<T, F> Node for FluentHandler<T, F>
  where T: FluentValue,
        F: Fn(Vec<AnyFluent>) -> T + Send
{
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
impl<T, F> FluentNode for FluentHandler<T, F>
  where T: FluentValue,
        F: Fn(Vec<AnyFluent>) -> T + Send
{
  async fn run(self, _request_tx: RequestTx) -> Result<()> {
    let (_node_tx, _node_rx) = match self.node_ch {
      Some((node_tx, node_rx)) => (node_tx, node_rx),
      None => bail!("FluentHandler '{}' not initialized, aborting", self.name),
    };

    Ok(())
  }
}
