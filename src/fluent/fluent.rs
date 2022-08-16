// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Key, Timestamp};

use eyre::{ensure, Result};
use getset::{CopyGetters, Getters, MutGetters};
use std::{cmp::Ordering, fmt};


#[derive(Clone, Debug, CopyGetters, Getters, MutGetters)]
/// Core application data type. Any property that is subject to change is
/// represented by a fluent.
pub struct Fluent<ValueType>
  where ValueType: fmt::Debug + Clone + PartialEq {
  #[getset(get = "pub", get_mut = "pub")]
  name:        String,
  #[getset(get = "pub", get_mut = "pub")]
  keys:        Vec<Key>,
  #[getset(get_copy = "pub", get_mut = "pub")]
  timestamp:   Timestamp,
  #[getset(get = "pub", get_mut = "pub")]
  value:       ValueType,
  #[getset(get_copy = "pub", get_mut = "pub")]
  last_change: Timestamp,
}

impl<ValueType> Fluent<ValueType>
  where ValueType: fmt::Debug + Clone + PartialEq
{
  /// A fluent needs to have a name and associated keys, i.e. sub-streams, to
  /// be uniquely identified. An initial timestamp and value must be provided,
  /// where the value can be of any type that implements `PartialEq + Eq`.
  pub fn new(name: &str,
             keys: &[Key],
             timestamp: Timestamp,
             value: &ValueType)
             -> Self {
    Self { name:        name.to_owned(),
           keys:        keys.to_owned(),
           timestamp:   timestamp,
           value:       value.clone(),
           last_change: timestamp, }
  }

  /// Update the fluent with a new timestamp and value. The new timestamp must
  /// be after the old one (tested via `timestamp > self.timestamp`) for this
  /// method to return `Ok(())`. This method also updates the value of
  /// `last_change` with the value of `timestamp` if the value is changed.
  pub fn update(&mut self,
                timestamp: Timestamp,
                value: ValueType)
                -> Result<()> {
    ensure!(timestamp > self.timestamp,
            "cannot update fluent with equal timestamp");

    self.timestamp = timestamp;
    if self.value != value {
      self.value = value;
      self.last_change = timestamp;
    }
    Ok(())
  }
}

impl<ValueType> PartialEq for Fluent<ValueType>
  where ValueType: fmt::Debug + Clone + PartialEq
{
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
    && self.keys == other.keys
    && self.timestamp == other.timestamp
  }
}

impl<ValueType> Eq for Fluent<ValueType>
  where ValueType: fmt::Debug + Clone + PartialEq
{
}

impl<ValueType> PartialOrd for Fluent<ValueType>
  where ValueType: fmt::Debug + Clone + PartialEq
{
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}

impl<ValueType> Ord for Fluent<ValueType>
  where ValueType: fmt::Debug + Clone + PartialEq
{
  fn cmp(&self, other: &Self) -> Ordering {
    self.timestamp.cmp(&other.timestamp)
  }
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::Fluent;

  use pretty_assertions::{assert_eq, assert_ne};
  use std::convert::From;


  #[test]
  fn textual_fluent_test() {
    let name = "textual_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = String::from("running");

    let fluent = Fluent::new(name, keys, timestamp, &value);

    assert_eq!(fluent.name, name.to_string());
    assert_eq!(fluent.name(), name);
    assert_eq!(fluent.keys, keys.to_vec());
    assert_eq!(fluent.keys(), keys);
    assert_eq!(fluent.timestamp, timestamp);
    assert_eq!(fluent.timestamp(), timestamp);
    assert_eq!(fluent.value, value);
    assert_eq!(fluent.value(), &value);
    assert_eq!(fluent.last_change, timestamp);
    assert_eq!(fluent.last_change(), timestamp);

    let mut other = Fluent::new(name, keys, timestamp, &value);
    assert_eq!(fluent, other);

    let new_timestamp = 1338;
    let new_value = String::from("blocked");

    assert!(other.update(timestamp, new_value.clone()).is_err());
    assert!(other.update(new_timestamp, new_value.clone()).is_ok());

    assert_eq!(other.value, new_value);
    assert_eq!(other.value(), &new_value);
    assert_eq!(other.last_change(), new_timestamp);
    assert!(fluent < other);

    assert!(other.update(1339, new_value).is_ok());
    assert_eq!(other.last_change(), new_timestamp);
    assert!(fluent < other);

    let mut other = fluent.clone();
    *other.name_mut() = "new_name".to_string();
    *other.keys_mut() = vec![42, 23];
    *other.timestamp_mut() = 1339;
    *other.value_mut() = "blocked".to_string();
    *other.last_change_mut() = 1338;

    assert_ne!(other, fluent);
  }

  #[test]
  fn integer_fluent_test() {
    let name = "integer_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = 3;

    let fluent = Fluent::new(name, keys, timestamp, &value);

    assert_eq!(fluent.name, name.to_string());
    assert_eq!(fluent.name(), name);
    assert_eq!(fluent.keys, keys.to_vec());
    assert_eq!(fluent.keys(), keys);
    assert_eq!(fluent.timestamp, timestamp);
    assert_eq!(fluent.timestamp(), timestamp);
    assert_eq!(fluent.value, value);
    assert_eq!(fluent.value(), &value);
    assert_eq!(fluent.last_change, timestamp);
    assert_eq!(fluent.last_change(), timestamp);

    let mut other = Fluent::new(name, keys, timestamp, &value);
    assert_eq!(fluent, other);

    let new_timestamp = 1338;
    let new_value = 2;

    assert!(other.update(timestamp, new_value).is_err());
    assert!(other.update(new_timestamp, new_value).is_ok());

    assert_eq!(other.value, new_value);
    assert_eq!(other.value(), &new_value);
    assert_eq!(other.last_change(), new_timestamp);
    assert!(fluent < other);

    assert!(other.update(1339, new_value).is_ok());
    assert_eq!(other.last_change(), new_timestamp);
    assert!(fluent < other);

    let mut other = fluent.clone();
    *other.name_mut() = "new_name".to_string();
    *other.keys_mut() = vec![42, 23];
    *other.timestamp_mut() = 1339;
    *other.value_mut() = 5;
    *other.last_change_mut() = 1338;

    assert_ne!(other, fluent);
  }

  #[test]
  fn floatpt_fluent_test() {
    let name = "floatpt_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = 3.14159;

    let fluent = Fluent::new(name, keys, timestamp, &value);

    assert_eq!(fluent.name, name.to_string());
    assert_eq!(fluent.name(), name);
    assert_eq!(fluent.keys, keys.to_vec());
    assert_eq!(fluent.keys(), keys);
    assert_eq!(fluent.timestamp, timestamp);
    assert_eq!(fluent.timestamp(), timestamp);
    assert_eq!(fluent.value, value);
    assert_eq!(fluent.value(), &value);
    assert_eq!(fluent.last_change, timestamp);
    assert_eq!(fluent.last_change(), timestamp);

    let mut other = Fluent::new(name, keys, timestamp, &value);
    assert_eq!(fluent, other);

    let new_timestamp = 1338;
    let new_value = 2.71828;

    assert!(other.update(timestamp, new_value).is_err());
    assert!(other.update(new_timestamp, new_value).is_ok());

    assert_eq!(other.value, new_value);
    assert_eq!(other.value(), &new_value);
    assert_eq!(other.last_change(), new_timestamp);
    assert!(fluent < other);

    assert!(other.update(1339, new_value).is_ok());
    assert_eq!(other.last_change(), new_timestamp);
    assert!(fluent < other);

    let mut other = fluent.clone();
    *other.name_mut() = "new_name".to_string();
    *other.keys_mut() = vec![42, 23];
    *other.timestamp_mut() = 1339;
    *other.value_mut() = 5.72;
    *other.last_change_mut() = 1338;

    assert_ne!(other, fluent);
  }

  #[test]
  fn boolean_fluent_test() {
    let name = "boolean_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = true;

    let fluent = Fluent::new(name, keys, timestamp, &value);

    assert_eq!(fluent.name, name.to_string());
    assert_eq!(fluent.name(), name);
    assert_eq!(fluent.keys, keys.to_vec());
    assert_eq!(fluent.keys(), keys);
    assert_eq!(fluent.timestamp, timestamp);
    assert_eq!(fluent.timestamp(), timestamp);
    assert_eq!(fluent.value, value);
    assert_eq!(fluent.value(), &value);
    assert_eq!(fluent.last_change, timestamp);
    assert_eq!(fluent.last_change(), timestamp);

    let mut other = Fluent::new(name, keys, timestamp, &value);
    assert_eq!(fluent, other);

    let new_timestamp = 1338;
    let new_value = false;

    assert!(other.update(timestamp, new_value).is_err());
    assert!(other.update(new_timestamp, new_value).is_ok());

    assert_eq!(other.value, new_value);
    assert_eq!(other.value(), &new_value);
    assert_eq!(other.last_change(), new_timestamp);
    assert!(fluent < other);

    assert!(other.update(1339, new_value).is_ok());
    assert_eq!(other.last_change(), new_timestamp);
    assert!(fluent < other);

    let mut other = fluent.clone();
    *other.name_mut() = "new_name".to_string();
    *other.keys_mut() = vec![42, 23];
    *other.timestamp_mut() = 1339;
    *other.value_mut() = false;
    *other.last_change_mut() = 1338;

    assert_ne!(other, fluent);
  }

  #[test]
  fn planept_fluent_test() {
    let name = "planept_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = (3.14159, 2.71828);

    let fluent = Fluent::new(name, keys, timestamp, &value);

    assert_eq!(fluent.name, name.to_string());
    assert_eq!(fluent.name(), name);
    assert_eq!(fluent.keys, keys.to_vec());
    assert_eq!(fluent.keys(), keys);
    assert_eq!(fluent.timestamp, timestamp);
    assert_eq!(fluent.timestamp(), timestamp);
    assert_eq!(fluent.value, value);
    assert_eq!(fluent.value(), &value);
    assert_eq!(fluent.last_change, timestamp);
    assert_eq!(fluent.last_change(), timestamp);

    let mut other = Fluent::new(name, keys, timestamp, &value);
    assert_eq!(fluent, other);

    let new_timestamp = 1338;
    let new_value = (2.71828, 3.14159);

    assert!(other.update(timestamp, new_value).is_err());
    assert!(other.update(new_timestamp, new_value).is_ok());

    assert_eq!(other.value, new_value);
    assert_eq!(other.value(), &new_value);
    assert_eq!(other.last_change(), new_timestamp);
    assert!(fluent < other);

    assert!(other.update(1339, new_value).is_ok());
    assert_eq!(other.last_change(), new_timestamp);
    assert!(fluent < other);

    let mut other = fluent.clone();
    *other.name_mut() = "new_name".to_string();
    *other.keys_mut() = vec![42, 23];
    *other.timestamp_mut() = 1339;
    *other.value_mut() = (5.72, 7.31);
    *other.last_change_mut() = 1338;

    assert_ne!(other, fluent);
  }
}
