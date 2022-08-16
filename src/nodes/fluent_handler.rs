// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{app_core::{util::unordered_congruent, RequestTx},
            fluent::{AnyFluent, EvalFn, Timestamp},
            nodes::{FluentNode, Node, NodeRx, NodeTx}};

use async_trait::async_trait;
use boolinator::Boolinator;
use derivative::Derivative;
use eyre::{bail, Result};
use std::collections::BTreeMap;
use tokio_stream::StreamExt;


#[derive(Derivative)]
#[derivative(Debug)]
/// A [`Node`] which handles the progression of fluents by subscribing to
/// dependency fluents and processing incoming values via a evaluation function
/// to produce an output fluent, publishing this [`AnyFluent`] to the
/// [`Broker`](crate::app_core::Broker).
pub struct FluentHandler {
  name:         String,
  dependencies: Vec<String>,
  #[derivative(Debug = "ignore")]
  eval_fn:      EvalFn,
  deps_buffer:  BTreeMap<Timestamp, Vec<AnyFluent>>,
  // history:                  Vec<AnyFluent>,
  // window:                   Option<usize>,
  node_ch:      Option<(NodeTx, NodeRx)>,
}

impl FluentHandler {
  /// Instantiate a [`FluentHandler`] with the name and dependencies of the
  /// [`AnyFluent`] it handles, as well as an evaluation function of type
  /// [`EvalFn`] (which is a wrapper struct for a closure).
  pub fn new(name: &str, dependencies: &[&str], eval_fn: EvalFn) -> Self {
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

impl Node for FluentHandler {
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
impl FluentNode for FluentHandler {
  /// This function contains a lot of the logic which defines the way fluents
  /// evolve over time, i.e. it contains the way dependencies are buffered,
  /// stored and detected to be complete for function evaluation, how
  /// background knowledge is obtained and provided to the evaluation function,
  /// how and when fluents are updated and published, all of that.
  async fn run(mut self: Box<Self>, _request_tx: RequestTx) -> Result<()> {
    let (node_tx, mut node_rx) = match self.node_ch {
      Some((node_tx, node_rx)) => (node_tx, node_rx),
      None => bail!("FluentHandler '{}' not initialized, aborting", self.name),
    };

    while let Some((_fluent_name, Ok(any_fluent))) = node_rx.next().await {
      // info!("FluentHandler '{}' received: {:?}", self.name, any_fluent);

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

      // we've got all the dependencies now - feed them into the eval_fn
      let fluent = AnyFluent::new(&self.name,
                                  &keys,
                                  timestamp,
                                  self.eval_fn.evaluate_with(deps));

      node_tx.send(fluent)?;
    }

    Ok(())
  }
}
