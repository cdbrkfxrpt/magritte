// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{FluentTrait, Key, Timestamp, ValueType};

use std::cmp::Ordering;


#[derive(Clone, Debug)]
/// Core application data type. Any property that is subject to change is
/// represented by a fluent.
pub struct InnerFluent<VT: ValueType + PartialEq + Clone> {
  name:        String,
  keys:        Vec<Key>,
  timestamp:   Timestamp,
  value:       VT,
  last_change: Timestamp,
}

impl<VT: ValueType + PartialEq + Clone> InnerFluent<VT> {
  /// A fluent needs to have a name and associated keys, i.e. sub-streams, to
  /// be uniquely identified. An initial timestamp and value must be provided,
  /// where the value can be of any type that implements `PartialEq + Eq`.
  pub fn new(name: &str,
             keys: &[Key],
             timestamp: Timestamp,
             value: VT)
             -> Self {
    Self { name: name.to_owned(),
           keys: keys.to_owned(),
           timestamp,
           value,
           last_change: timestamp }
  }

  /// Update the fluent with a new timestamp. Update value if it has
  /// changed, and if the value is updated, also `last_change` is updated.
  pub fn update(&mut self, timestamp: Timestamp, value: VT) {
    self.timestamp = timestamp;
    if self.value != value {
      self.value = value;
      self.last_change = timestamp;
    }
  }
}

impl<VT: ValueType + PartialEq + Clone> FluentTrait for InnerFluent<VT> {
  fn name(&self) -> &str {
    &self.name
  }

  fn keys(&self) -> &[Key] {
    self.keys.as_slice()
  }

  fn timestamp(&self) -> Timestamp {
    self.timestamp
  }

  fn boxed_value(&self) -> Box<dyn ValueType> {
    Box::new(self.value.clone())
  }

  fn last_change(&self) -> Timestamp {
    self.last_change
  }
}

impl<VT: ValueType + PartialEq + Clone> PartialEq for InnerFluent<VT> {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
    && self.keys == other.keys
    && self.timestamp == other.timestamp
  }
}

impl<VT: ValueType + PartialEq + Clone> Eq for InnerFluent<VT> {}

impl<VT: ValueType + PartialEq + Clone> PartialOrd for InnerFluent<VT> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl<VT: ValueType + PartialEq + Clone> Ord for InnerFluent<VT> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.keys.cmp(&other.keys)
  }
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::{FluentTrait, InnerFluent};

  use pretty_assertions::assert_eq;
  use std::{convert::From, f64::consts};


  #[test]
  fn textual_fluent_test() {
    let name = "textual_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = String::from("running");

    let fluent = InnerFluent::new(name, keys, timestamp, value.clone());

    assert_eq!(fluent.name, name.to_string());
    assert_eq!(fluent.name(), name);
    assert_eq!(fluent.keys, keys.to_vec());
    assert_eq!(fluent.keys(), keys);
    assert_eq!(fluent.timestamp, timestamp);
    assert_eq!(fluent.timestamp(), timestamp);
    assert_eq!(fluent.value, value);
    let boxed_value = fluent.boxed_value().downcast::<String>().unwrap();
    assert_eq!(*boxed_value, value);
    assert_eq!(fluent.last_change, timestamp);
    assert_eq!(fluent.last_change(), timestamp);
  }

  #[test]
  fn integer_fluent_test() {
    let name = "integer_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = 3;

    let fluent = InnerFluent::new(name, keys, timestamp, value);

    assert_eq!(fluent.name, name.to_string());
    assert_eq!(fluent.name(), name);
    assert_eq!(fluent.keys, keys.to_vec());
    assert_eq!(fluent.keys(), keys);
    assert_eq!(fluent.timestamp, timestamp);
    assert_eq!(fluent.timestamp(), timestamp);
    assert_eq!(fluent.value, value);
    let boxed_value = fluent.boxed_value().downcast::<i64>().unwrap();
    assert_eq!(*boxed_value, value);
    assert_eq!(fluent.last_change, timestamp);
    assert_eq!(fluent.last_change(), timestamp);
  }

  #[test]
  fn floatpt_fluent_test() {
    let name = "floatpt_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = consts::PI;

    let fluent = InnerFluent::new(name, keys, timestamp, value);

    assert_eq!(fluent.name, name.to_string());
    assert_eq!(fluent.name(), name);
    assert_eq!(fluent.keys, keys.to_vec());
    assert_eq!(fluent.keys(), keys);
    assert_eq!(fluent.timestamp, timestamp);
    assert_eq!(fluent.timestamp(), timestamp);
    assert_eq!(fluent.value, value);
    let boxed_value = fluent.boxed_value().downcast::<f64>().unwrap();
    assert_eq!(*boxed_value, value);
    assert_eq!(fluent.last_change, timestamp);
    assert_eq!(fluent.last_change(), timestamp);
  }

  #[test]
  fn boolean_fluent_test() {
    let name = "boolean_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = true;

    let fluent = InnerFluent::new(name, keys, timestamp, value);

    assert_eq!(fluent.name, name.to_string());
    assert_eq!(fluent.name(), name);
    assert_eq!(fluent.keys, keys.to_vec());
    assert_eq!(fluent.keys(), keys);
    assert_eq!(fluent.timestamp, timestamp);
    assert_eq!(fluent.timestamp(), timestamp);
    assert_eq!(fluent.value, value);
    let boxed_value = fluent.boxed_value().downcast::<bool>().unwrap();
    assert_eq!(*boxed_value, value);
    assert_eq!(fluent.last_change, timestamp);
    assert_eq!(fluent.last_change(), timestamp);
  }

  #[test]
  fn planept_fluent_test() {
    let name = "planept_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = (consts::PI, consts::E);

    let fluent = InnerFluent::new(name, keys, timestamp, value);

    assert_eq!(fluent.name, name.to_string());
    assert_eq!(fluent.name(), name);
    assert_eq!(fluent.keys, keys.to_vec());
    assert_eq!(fluent.keys(), keys);
    assert_eq!(fluent.timestamp, timestamp);
    assert_eq!(fluent.timestamp(), timestamp);
    assert_eq!(fluent.value, value);
    let boxed_value = fluent.boxed_value().downcast::<(f64, f64)>().unwrap();
    assert_eq!(*boxed_value, value);
    assert_eq!(fluent.last_change, timestamp);
    assert_eq!(fluent.last_change(), timestamp);
  }
}
