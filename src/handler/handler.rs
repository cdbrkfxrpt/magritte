// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::EvalFn;
use crate::{app_core::{Database, Node, NodeRx, NodeTx},
            fluent::{Fluent, FluentTrait, Key}};

use async_trait::async_trait;
use derivative::Derivative;
use eyre::{bail, Result};
use std::{collections::BTreeMap,
          sync::{Arc, Mutex}};
use tokio_stream::StreamExt;
use tracing::debug;


#[derive(Debug, PartialEq, Eq)]
pub enum KeyDependency {
  Static,
  Concurrent,
  NonConcurrent { timeout: usize },
}


#[derive(Derivative)]
#[derivative(Debug)]
/// Allows for ergonomic definition of [`Handler`]s.
pub struct HandlerDefinition<'a> {
  pub fluent_name:    &'a str,
  pub dependencies:   &'a [&'a str],
  pub key_dependency: KeyDependency,
  pub database_query: Option<&'a str>,
  #[derivative(Debug = "ignore")]
  pub eval_fn:        EvalFn,
}


#[derive(Derivative)]
#[derivative(Debug)]
/// A [`Node`] which handles the progression of fluents by subscribing to
/// dependency fluents and processing incoming values via a evaluation function
/// to produce an output fluent, publishing this [`Fluent`] to the `Broker`.
pub struct Handler {
  fluent_name:    String,
  dependencies:   Vec<String>,
  key_dependency: KeyDependency,
  #[derivative(Debug = "ignore")]
  eval_fn:        EvalFn,
  database:       Database,
  deps_buffer:    BTreeMap<Vec<Key>, Vec<Fluent>>,
  buffer_timeout: usize,
  node_ch:        Option<(NodeTx, NodeRx)>,
}

impl Handler {
  /// Instantiate a [`Handler`] with the name and dependencies of the
  /// [`Fluent`] it handles, as well as an evaluation function of type
  /// [`EvalFn`] (which is a wrapper struct for a closure).
  pub async fn new(def: HandlerDefinition<'_>,
                   buffer_timeout: usize,
                   database: Database)
                   -> Result<Handler> {
    let fluent_name = def.fluent_name.to_owned();
    let dependencies = def.dependencies
                          .iter()
                          .map(|e| e.to_string())
                          .collect::<Vec<_>>();
    let key_dependency = def.key_dependency;
    let eval_fn = def.eval_fn;
    let database = database.with_template_option(def.database_query);
    let deps_buffer = BTreeMap::new();
    let node_ch = None;

    Ok(Self { fluent_name,
              dependencies,
              key_dependency,
              eval_fn,
              database,
              deps_buffer,
              buffer_timeout,
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
      None => {
        bail!("Handler '{}' not initialized, aborting", self.fluent_name)
      }
    };

    let fluent_name = self.fluent_name;
    let eval_fn = self.eval_fn.into_inner();
    let database = self.database;
    let history = Arc::new(Mutex::new(Vec::<Fluent>::new()));

    while let Some((name, Ok(fluent))) = node_rx.next().await {
      let keys = fluent.keys().to_vec();
      let timestamp = fluent.timestamp();
      let history_mtx = history.clone();

      // if we have a static key dependency - in other words, if this value
      // never changes for one key and thus needs to be calculated only once -
      // look it up in the history and if we have it, return it from there with
      // an updated timestamp and skip the remainder of the loop
      if self.key_dependency == KeyDependency::Static {
        // unwrap here is safe: locking `Mutex` cannot fail
        let mut history = history_mtx.lock().unwrap();
        if let Some(history_fluent) =
          history.iter_mut().find(|f| f.keys() == keys)
        {
          debug!("updating and sending '{}' from history", fluent_name);
          history_fluent.update(timestamp, history_fluent.boxed_value());
          if let Err(err) = node_tx.send(history_fluent.clone()) {
            eprintln!("error sending to broker: {}", err);
          }
          continue;
        }
      }

      // check if the dependency buffer has this key (combination) already
      match self.deps_buffer.get_mut(&keys) {
        // if yes...
        Some(buffer) => {
          // ... check if we have this fluent (by name) already and...
          if let Some(buffered_fluent) =
            buffer.iter_mut().find(|f| f.name() == name)
          {
            // ... if yes, update it.
            buffered_fluent.update(timestamp, fluent.boxed_value())
          } else {
            // ... if not, push it into the buffer.
            buffer.push(fluent)
          }
        }
        // if not, add new entry for this key (combination) to the buffer with
        // the fluent inside
        None => {
          self.deps_buffer.insert(keys.clone(), vec![fluent]);
        }
      }

      // before we do anything, we prune the buffer of old fluents
      util::prune_buffer(&mut self.deps_buffer,
                         timestamp,
                         self.buffer_timeout);

      // assemble dependency sets to run through
      let dependency_sets = util::dependency_sets(&mut self.deps_buffer,
                                                  &keys,
                                                  timestamp,
                                                  &self.dependencies,
                                                  &self.key_dependency);
      debug!("{:24} dependency sets: {:?}", fluent_name, dependency_sets);
      // continue;

      for (dep_keys, dependencies) in dependency_sets.into_iter() {
        let fluent_name = fluent_name.clone();
        let node_tx = node_tx.clone();
        let database = database.clone();
        let eval_fn = eval_fn.clone();
        let history_mtx = history_mtx.clone();

        tokio::spawn(async move {
          // we've got all the dependencies now - feed them into the eval_fn
          let value = match eval_fn(dependencies.clone(), database).await {
            Some(value) => value,
            None => return,
          };

          let mut history = history_mtx.lock().unwrap();
          let fluent = match history.iter_mut().find(|f| f.keys() == dep_keys)
          {
            Some(fluent) => {
              fluent.update(timestamp, value);
              fluent.clone()
            }
            None => {
              let fluent =
                Fluent::new(&fluent_name, &dep_keys, timestamp, value);
              history.push(fluent.clone());
              fluent
            }
          };

          if let Err(err) = node_tx.send(fluent) {
            eprintln!("unable to send fluent to broker: {}", err);
          }
        });
      }
    }
    Ok(())
  }
}

#[async_trait]
impl Node for Handler {
  fn publishes(&self) -> Vec<String> {
    vec![self.fluent_name.clone()]
  }

  fn subscribes_to(&self) -> Vec<String> {
    self.dependencies.clone()
  }

  fn initialize(&mut self, node_tx: NodeTx, node_rx: NodeRx) {
    self.node_ch = Some((node_tx, node_rx));
  }
}

mod util {
  use super::KeyDependency;
  use crate::fluent::{Fluent, FluentTrait, Key, Timestamp};

  use array_tool::vec::{Union, Uniq};
  use itertools::Itertools;
  use std::collections::BTreeMap;


  /// Merges two key lists, removing duplicates.
  fn merge<T>(lhs: &[T], rhs: &[T]) -> Vec<T>
    where T: PartialEq + Ord + Clone {
    lhs.to_vec()
       .union(rhs.to_vec())
       .into_iter()
       .sorted()
       .collect()
  }

  /// Checks if all [`Fluent`]s in collection have the same timestamp.
  fn same_timestamps(fluents: &[Fluent]) -> bool {
    fluents.windows(2)
           .all(|w| w[0].timestamp() == w[1].timestamp())
  }

  /// Get names of all [`Fluent`]s in collection.
  fn fluent_names(fluents: &[Fluent]) -> Vec<String> {
    fluents.iter().map(|f| f.name().to_string()).collect()
  }

  /// Get keys of all  [`Fluent`]s in collection.
  pub fn fluent_keys(fluents: &[Fluent]) -> Vec<Key> {
    fluents.iter()
           .flat_map(|f| f.keys().to_vec())
           .sorted()
           .collect::<Vec<_>>()
           .unique()
  }

  /// Check if two collections ([`Vec`]s) have the same elements.
  pub fn equal<T: PartialEq>(lhs: &[T], rhs: &[T]) -> bool {
    lhs.len() == rhs.len() && lhs.iter().all(|e| rhs.contains(e))
  }

  /// Sorts a `Vec` of [`Fluent`]s by name, in a given order of names.
  pub fn sort_by_given_order(fluents: &mut [Fluent], order: &[String]) {
    fluents.sort_by(|lhs, rhs| {
             let lhs_pos =
               order.iter().position(|name| name == lhs.name()).unwrap();
             let rhs_pos =
               order.iter().position(|name| name == rhs.name()).unwrap();
             lhs_pos.cmp(&rhs_pos)
           });
  }

  /// Assort the set of dependencies from the buffer.
  pub fn dependency_sets(buffer: &mut BTreeMap<Vec<Key>, Vec<Fluent>>,
                         keys: &Vec<Key>,
                         timestamp: Timestamp,
                         dependencies: &[String],
                         key_dependency: &KeyDependency)
                         -> BTreeMap<Vec<Key>, Vec<Fluent>> {
    let mut dependency_sets = BTreeMap::new();
    let mut key_dependencies = buffer[keys].clone();
    match key_dependency {
      KeyDependency::Static | KeyDependency::Concurrent => {
        if equal(&fluent_names(&key_dependencies).unique(), dependencies)
           && same_timestamps(&key_dependencies)
        {
          sort_by_given_order(&mut key_dependencies, dependencies);
          dependency_sets.insert(fluent_keys(&key_dependencies),
                                 key_dependencies);
          buffer.remove(keys);
        }
      }
      KeyDependency::NonConcurrent { timeout } => {
        prune_buffer(buffer, timestamp, *timeout);
        for (rhs_keys, fluents) in buffer.iter() {
          if rhs_keys != keys {
            dependency_sets.insert(merge(keys, rhs_keys),
                                   merge(&key_dependencies, fluents));
          }
        }
      }
    }
    dependency_sets
  }

  /// Removes all fluents from buffer with timestamps older than
  /// `timestamp.saturating_sub(timeout)` (see [`usize`
  /// docs](https://doc.rust-lang.org/std/primitive.usize.html) for information
  /// on `saturating_sub`).
  pub fn prune_buffer(buffer: &mut BTreeMap<Vec<Key>, Vec<Fluent>>,
                      timestamp: Timestamp,
                      timeout: usize) {
    let cutoff = timestamp.saturating_sub(timeout);
    for fluents in buffer.values_mut() {
      fluents.retain(|f| f.timestamp() > cutoff);
    }
  }
}

// fin --------------------------------------------------------------------- //
