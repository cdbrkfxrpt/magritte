// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{app_core::RequestTx,
            fluent::{AnyFluent, Timestamp},
            nodes::{FluentNode, Node, NodeRx, NodeTx}};

use async_trait::async_trait;
use eyre::{bail, Result};
use std::collections::BTreeMap;


#[derive(Debug)]
pub struct FluentHandler<F>
  where F: Fn(Vec<AnyFluent>, Option<RequestTx>) -> AnyFluent + Send {
  name:         String,
  dependencies: Vec<String>,
  eval_fn:      F,
  request_tx:   RequestTx,
  deps_buffer:  BTreeMap<Timestamp, Vec<AnyFluent>>,
  // history:                  Vec<AnyFluent>,
  // window:                   Option<usize>,
  node_ch:      Option<(NodeTx, NodeRx)>,
}

impl<F> FluentHandler<F>
  where F: Fn(Vec<AnyFluent>, Option<RequestTx>) -> AnyFluent + Send
{
  pub fn new(name: &str,
             dependencies: &[&str],
             eval_fn: F,
             request_tx: RequestTx)
             -> Self {
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
           request_tx,
           node_ch }
  }
}

impl<F> Node for FluentHandler<F>
  where F: Fn(Vec<AnyFluent>, Option<RequestTx>) -> AnyFluent + Send
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
impl<F> FluentNode for FluentHandler<F>
  where F: Fn(Vec<AnyFluent>, Option<RequestTx>) -> AnyFluent + Send
{
  async fn run(self) -> Result<()> {
    let (_node_tx, _node_rx) = match self.node_ch {
      Some((node_tx, node_rx)) => (node_tx, node_rx),
      None => bail!("FluentHandler '{}' not initialized, aborting", self.name),
    };

    Ok(())
  }
}
