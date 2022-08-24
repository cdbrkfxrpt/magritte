// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Context, EvalFn};
use crate::{fluent::{Fluent, FluentTrait, Key},
            nodes::{Node, NodeRx, NodeTx}};

use async_trait::async_trait;
use derivative::Derivative;
use eyre::{bail, Result};
use std::collections::BTreeMap;
use tokio_postgres::Client;
use tokio_stream::StreamExt;
use tracing::info;


#[derive(Debug)]
pub enum KeyDependency {
  Concurrent,
  NonConcurrent,
}


#[derive(Derivative)]
#[derivative(Debug)]
/// Allows for ergonomic definition of [`FluentHandler`]s.
pub struct FluentHandlerDefinition<'a> {
  pub fluent_name:    &'a str,
  pub dependencies:   &'a [&'a str],
  pub key_dependency: KeyDependency,
  pub database_query: Option<&'a str>,
  #[derivative(Debug = "ignore")]
  pub eval_fn:        EvalFn<'a>,
  pub prune_after:    usize,
}


#[derive(Derivative)]
#[derivative(Debug)]
/// A [`Node`] which handles the progression of fluents by subscribing to
/// dependency fluents and processing incoming values via a evaluation function
/// to produce an output fluent, publishing this [`Fluent`] to the
/// [`Broker`](crate::app_core::Broker).
pub struct FluentHandler<'a> {
  fluent_name:    String,
  dependencies:   Vec<String>,
  key_dependency: KeyDependency,
  context:        Context,
  #[derivative(Debug = "ignore")]
  eval_fn:        EvalFn<'a>,
  deps_buffer:    BTreeMap<Vec<Key>, Vec<Fluent>>,
  prune_after:    usize,
  history:        Vec<Fluent>,
  // window:                   Option<usize>,
  node_ch:        Option<(NodeTx, NodeRx)>,
}

impl<'a> FluentHandler<'a> {
  /// Instantiate a [`FluentHandler`] with the name and dependencies of the
  /// [`Fluent`] it handles, as well as an evaluation function of type
  /// [`EvalFn`] (which is a wrapper struct for a closure).
  pub async fn new(def: FluentHandlerDefinition<'a>,
                   database_client: Client)
                   -> Result<FluentHandler> {
    let fluent_name = def.fluent_name.to_owned();
    let dependencies = def.dependencies
                          .iter()
                          .map(|e| e.to_string())
                          .collect::<Vec<_>>();
    let key_dependency = def.key_dependency;
    let context = Context::new(database_client, def.database_query).await?;
    let eval_fn = def.eval_fn;
    let deps_buffer = BTreeMap::new();
    let prune_after = def.prune_after;
    let history = Vec::new();
    let node_ch = None;

    Ok(Self { fluent_name,
              dependencies,
              key_dependency,
              context,
              eval_fn,
              deps_buffer,
              prune_after,
              history,
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
      None => bail!("FluentHandler '{}' not initialized, aborting",
                    self.fluent_name),
    };

    let eval_fn = self.eval_fn.into_inner();
    let context = self.context.arc_ptr();

    while let Some((name, Ok(fluent))) = node_rx.next().await {
      let keys = fluent.keys().to_vec();
      let timestamp = fluent.timestamp();

      // check if the dependency buffer has this key (combination) already
      match self.deps_buffer.get_mut(&keys) {
        // if yes...
        Some(buffer) => {
          // ... check if we have this fluent (by name) already and...
          if let Some(fluent) = buffer.iter_mut().find(|f| f.name() == name) {
            // ... if yes, update it.
            fluent.update(timestamp, fluent.boxed_value())
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

      //
      let dependency_sets = util::dependency_sets(&self.deps_buffer,
                                                  &keys,
                                                  &self.dependencies,
                                                  &self.key_dependency);
      info!("{:24} dependency sets: {:?}",
            self.fluent_name, dependency_sets);
      // continue;

      for (dep_keys, dependencies) in dependency_sets.into_iter() {
        // we've got all the dependencies now - feed them into the eval_fn
        let value = match eval_fn(dependencies.clone(), context.clone()).await
        {
          Some(value) => value,
          None => continue,
        };

        let fluent =
          match self.history.iter_mut().find(|f| f.keys() == dep_keys) {
            Some(fluent) => {
              fluent.update(timestamp, value);
              fluent.clone()
            }
            None => {
              let fluent =
                Fluent::new(&self.fluent_name, &dep_keys, timestamp, value);
              self.history.push(fluent.clone());
              fluent
            }
          };
        node_tx.send(fluent)?;
      }
    }
    Ok(())
  }
}

#[async_trait]
impl<'a> Node for FluentHandler<'a> {
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
  use crate::{fluent::{Fluent, FluentTrait, Key},
              nodes::fluent_handler::KeyDependency};

  use array_tool::vec::{Intersect, Union, Uniq};
  use boolinator::Boolinator;
  use itertools::Itertools;
  use std::collections::{BTreeMap, BTreeSet};
  // use tracing::info;


  /// Checks if two key sets overlap, i.e. have at least one common element.
  pub fn have_overlap(lhs: &Vec<Key>, rhs: &Vec<Key>) -> bool {
    !lhs.intersect(rhs.clone()).is_empty()
  }

  /// Merges two key lists, removing duplicates.
  pub fn merge<T>(lhs: &Vec<T>, rhs: &Vec<T>) -> Vec<T>
    where T: PartialEq + Ord + Clone {
    lhs.union(rhs.clone()).into_iter().sorted().collect()
  }

  /// Merges two key lists if they have common elements, removing duplicates.
  pub fn merge_if_overlap(lhs: &Vec<Key>, rhs: &Vec<Key>) -> Option<Vec<Key>> {
    if !have_overlap(lhs, rhs) {
      return None;
    }
    Some(merge(lhs, rhs))
  }

  /// Finds the longest sets containing a given smaller set of keys.
  pub fn key_matches(key_sets: &Vec<Vec<Key>>,
                     keys: &Vec<Key>)
                     -> Vec<Vec<Key>> {
    let mut key_matches = BTreeSet::new();
    let mut max_len = 0;
    for key_set in key_sets.iter() {
      if let Some(key_match) = merge_if_overlap(key_set, keys) {
        max_len = key_match.len();
        key_matches.insert(key_match);
      }
    }
    key_matches.retain(|m| m.len() == max_len);
    key_matches.into_iter().collect()
  }

  /// Checks if all [`Fluent`]s in collection have the same timestamp.
  pub fn same_timestamps(fluents: &Vec<Fluent>) -> bool {
    fluents.windows(2)
           .all(|w| w[0].timestamp() == w[1].timestamp())
  }

  /// Get names of all [`Fluent`]s in collection.
  pub fn fluent_names(fluents: &Vec<Fluent>) -> Vec<String> {
    fluents.iter().map(|f| f.name().to_string()).collect()
  }

  /// Get keys of all  [`Fluent`]s in collection.
  pub fn fluent_keys(fluents: &Vec<Fluent>) -> Vec<Key> {
    fluents.iter()
           .map(|f| f.keys().iter().cloned().collect::<Vec<_>>())
           .flatten()
           .sorted()
           .collect::<Vec<_>>()
           .unique()
  }

  /// Check if two collections ([`Vec`]s) have the same elements.
  pub fn equal<T: PartialEq>(lhs: &Vec<T>, rhs: &Vec<T>) -> bool {
    lhs.len() == rhs.len() && lhs.iter().all(|e| rhs.contains(&e))
  }

  /// Sorts a `Vec` of [`Fluent`]s by name, in a given order of names.
  pub fn sort_by_given_order(fluents: &mut Vec<Fluent>, order: &Vec<String>) {
    fluents.sort_by(|lhs, rhs| {
             let lhs_pos =
               order.iter().position(|name| name == lhs.name()).unwrap();
             let rhs_pos =
               order.iter().position(|name| name == rhs.name()).unwrap();
             lhs_pos.cmp(&rhs_pos)
           });
  }

  /// Other stuff
  pub fn dependency_sets(buffer: &BTreeMap<Vec<Key>, Vec<Fluent>>,
                         keys: &Vec<Key>,
                         dependencies: &Vec<String>,
                         key_dependency: &KeyDependency)
                         -> BTreeMap<Vec<Key>, Vec<Fluent>> {
    let mut dependency_sets = BTreeMap::new();
    match key_dependency {
      KeyDependency::Concurrent => {
        let key_sets = buffer.keys().cloned().collect();
        for key_match in key_matches(&key_sets, keys) {
          // iterate through buffer, returning those fluent sets the keys of
          // which have an overlap with the key match (so a fluent with the key
          // [23] is collected same as one with the key [23, 42] if the key
          // match is [23, 42]) and collecting them into a `Vec`
          #[rustfmt::skip]
          let mut key_deps =
            buffer.iter()
                  .filter_map(|(keys, fluents)| {
                    (
                      have_overlap(keys, &key_match)
                      && same_timestamps(fluents)
                    ).as_some(fluents.clone())
                  })
                  .flatten()
                  .collect::<Vec<_>>();

          // check if all dependencies are present and if so, sort them in the
          // same order as the dependency list and insert them into the
          // dependency set map
          if equal(&fluent_names(&key_deps).unique(), dependencies) {
            sort_by_given_order(&mut key_deps, dependencies);
            dependency_sets.insert(fluent_keys(&key_deps), key_deps);
          }
        }

        // deps: dependencies filtered by key as Vec
        // if !matching(&get_names(&deps), &self.dependencies) {
        //   continue;
        // }
        //
        // now that we've checked that deps contains all elements required by
        // self.dependencies, we can safely unwrap the position calls here:
        // util::sort_fluents_by_given_order(&mut deps, &self.dependencies);
      }
      KeyDependency::NonConcurrent => {
        let key_dependencies = buffer[keys].clone();
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
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::util;

  use pretty_assertions::assert_eq;
  // use std::collections::BTreeSet;

  #[test]
  fn merge_if_overlap_test() {
    let keys_a = vec![42];
    let keys_b = vec![23, 42];
    assert_eq!(util::merge_if_overlap(&keys_a, &keys_b).unwrap(), keys_b);
    let keys_a = vec![23, 42];
    let keys_b = vec![23, 42];
    assert_eq!(util::merge_if_overlap(&keys_a, &keys_b).unwrap(), keys_b);

    let keys_a = vec![42];
    let keys_b = vec![42, 23];
    let keys_c = vec![23, 42];
    assert_eq!(util::merge_if_overlap(&keys_a, &keys_b).unwrap(), keys_c);

    let keys_a = vec![42];
    let keys_b = vec![42];
    assert_eq!(util::merge_if_overlap(&keys_a, &keys_b).unwrap(), keys_b);

    let keys_a = vec![42];
    let keys_b = vec![23];
    assert!(util::merge_if_overlap(&keys_a, &keys_b).is_none());

    let keys_a = vec![];
    let keys_b = vec![23, 42];
    assert!(util::merge_if_overlap(&keys_a, &keys_b).is_none());
  }

  #[test]
  fn key_matches_test() {
    let key_sets = vec![vec![23], vec![42], vec![72], vec![94]];
    let keys = vec![72];
    let key_matches = vec![vec![72]];
    assert_eq!(util::key_matches(&key_sets, &keys), key_matches);

    let key_sets = vec![vec![23],     // e.g. rendez_vous_conditions
                        vec![42],     // e.g. rendez_vous_conditions
                        vec![23, 42], // e.g. proximity
                        vec![72],
                        vec![94],
                        vec![42, 72]];

    let keys = vec![23];
    let key_matches = vec![vec![23, 42]];
    assert_eq!(util::key_matches(&key_sets, &keys), key_matches);
    // with the above key_matches, we can now retrieve the proximity and both
    // the rendez_vous_conditions by simply iterating the buffer and comparing
    // keys / checking if key is contained in key_matches.
    //
    // finally, sort by dependency list order. done.

    let keys = vec![42];
    let key_matches = vec![vec![23, 42], vec![42, 72]];
    assert_eq!(util::key_matches(&key_sets, &keys), key_matches);

    let keys = vec![94];
    let key_matches = vec![vec![94]];
    assert_eq!(util::key_matches(&key_sets, &keys), key_matches);

    let keys = vec![1337];
    assert!(util::key_matches(&key_sets, &keys).is_empty());

    let keys = vec![23, 42];
    let key_sets = vec![vec![23], vec![23, 42, 72], vec![94], vec![42, 72]];
    let key_matches = vec![vec![23, 42, 72]];
    assert_eq!(util::key_matches(&key_sets, &keys), key_matches);
  }
}
