// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Context, EvalFn};
use crate::{app_core::util::unordered_congruent,
            fluent::{Fluent, FluentTrait, Timestamp},
            nodes::{Node, NodeRx, NodeTx}};

use async_trait::async_trait;
use boolinator::Boolinator;
use derivative::Derivative;
use eyre::{bail, Result};
use std::collections::BTreeMap;
use tokio_postgres::Client;
use tokio_stream::StreamExt;


#[derive(Derivative)]
#[derivative(Debug)]
/// Allows for ergonomic definition of [`FluentHandler`]s.
pub struct FluentHandlerDefinition<'a> {
  pub name:           &'a str,
  pub dependencies:   &'a [&'a str],
  pub database_query: Option<&'a str>,
  #[derivative(Debug = "ignore")]
  pub eval_fn:        EvalFn<'a>,
}


#[derive(Derivative)]
#[derivative(Debug)]
/// A [`Node`] which handles the progression of fluents by subscribing to
/// dependency fluents and processing incoming values via a evaluation function
/// to produce an output fluent, publishing this [`Fluent`] to the
/// [`Broker`](crate::app_core::Broker).
pub struct FluentHandler<'a> {
  name:         String,
  dependencies: Vec<String>,
  context:      Context,
  #[derivative(Debug = "ignore")]
  eval_fn:      EvalFn<'a>,
  deps_buffer:  BTreeMap<Timestamp, Vec<Fluent>>,
  // history:                  Vec<Fluent>,
  // window:                   Option<usize>,
  node_ch:      Option<(NodeTx, NodeRx)>,
}

impl<'a> FluentHandler<'a> {
  /// Instantiate a [`FluentHandler`] with the name and dependencies of the
  /// [`Fluent`] it handles, as well as an evaluation function of type
  /// [`EvalFn`] (which is a wrapper struct for a closure).
  pub async fn new(def: FluentHandlerDefinition<'a>,
                   database_client: Client)
                   -> Result<FluentHandler> {
    let name = def.name.to_owned();
    let dependencies = def.dependencies
                          .iter()
                          .map(|e| e.to_string())
                          .collect::<Vec<_>>();
    let context = Context::new(database_client, def.database_query).await?;
    let eval_fn = def.eval_fn;
    let deps_buffer = BTreeMap::new();
    let node_ch = None;

    Ok(Self { name,
              dependencies,
              context,
              eval_fn,
              deps_buffer,
              node_ch })
  }

  /// This function contains a lot of the logic which defines the way fluents
  /// evolve over time, i.e. it contains the way dependencies are buffered,
  /// stored and detected to be complete for function evaluation, how
  /// background knowledge is obtained and provided to the evaluation function,
  /// how and when fluents are updated and published, all of that.
  pub async fn run(mut self) -> Result<()> {
    let (node_tx, node_rx) = match &mut self.node_ch {
      Some((node_tx, node_rx)) => (node_tx, node_rx),
      None => bail!("FluentHandler '{}' not initialized, aborting", self.name),
    };

    let eval_fn = self.eval_fn.into_inner();
    let context = self.context.arc_ptr();

    while let Some((_fluent_name, Ok(any_fluent))) = node_rx.next().await {
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
                                    .collect::<Vec<Fluent>>();

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
      let value = eval_fn(deps, context.clone()).await;

      node_tx.send(Fluent::new(&self.name, &keys, timestamp, value))?;
    }
    Ok(())
  }
}

#[async_trait]
impl<'a> Node for FluentHandler<'a> {
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
