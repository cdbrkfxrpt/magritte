// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use eyre::{ensure, Result};
use getset::{CopyGetters, Getters, MutGetters};
use std::cmp::Ordering;


/// Type alias for key, i.e. sub-stream identifier, type.
pub type Key = usize;
/// Type alias for timestamp type.
pub type Timestamp = usize;


#[derive(Clone, Debug, PartialEq, Eq, CopyGetters, Getters, MutGetters)]
/// Core application data type. Any property that is subject to change is
/// represented by a fluent.
pub struct Fluent<ValueType: PartialEq + Eq> {
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

impl<ValueType: PartialEq + Eq> Fluent<ValueType> {
  /// A fluent needs to have a name and associated keys, i.e. sub-streams, to
  /// be uniquely identified. An initial timestamp and value must be provided,
  /// where the value can be of any type that implements `PartialEq + Eq`.
  pub fn new(name: &str,
             keys: &[Key],
             timestamp: Timestamp,
             value: ValueType)
             -> Self {
    Self { name:        name.to_owned(),
           keys:        keys.to_owned(),
           timestamp:   timestamp,
           value:       value,
           last_change: timestamp, }
  }

  /// Update the fluent with a new timestamp and value. The new timestamp must
  /// be after the old one (tested via `timestamp > self.timestamp`) for this
  /// method to return `Ok(())`. This method also updates the value of
  /// `last_change` if the value is changed.
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

impl<ValueType: PartialEq + Eq> PartialOrd for Fluent<ValueType> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}

impl<ValueType: PartialEq + Eq> Ord for Fluent<ValueType> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.timestamp.cmp(&other.timestamp)
  }
}


/// Enables the use of `Fluent`s as `dyn AnyFluent` objects.
pub trait AnyFluent {}
impl<ValueType: PartialEq + Eq> AnyFluent for Fluent<ValueType> {}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::{AnyFluent, Fluent};
  use float_ord::FloatOrd;
  use pretty_assertions::assert_eq;
  use static_assertions::assert_impl_all;
  use std::convert::From;

  #[test]
  fn textual_fluent_test() {
    assert_impl_all!(Fluent<String>: AnyFluent);

    let name = "numeric_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = String::from("running");

    let fluent = Fluent::new(name, keys, timestamp, value.clone());

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

    let mut other = Fluent::new(name, keys, timestamp, value);
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
  }

  #[test]
  fn integer_fluent_test() {
    assert_impl_all!(Fluent<i32>: AnyFluent);

    let name = "numeric_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = 3;

    let fluent = Fluent::new(name, keys, timestamp, value);

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

    let mut other = Fluent::new(name, keys, timestamp, value);
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
  }

  #[test]
  fn float_fluent_test() {
    assert_impl_all!(Fluent<FloatOrd<f32>>: AnyFluent);

    let name = "numeric_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = FloatOrd(3.14159);

    let fluent = Fluent::new(name, keys, timestamp, value);

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

    let mut other = Fluent::new(name, keys, timestamp, value);
    assert_eq!(fluent, other);

    let new_timestamp = 1338;
    let new_value = FloatOrd(2.71828);

    assert!(other.update(timestamp, new_value).is_err());
    assert!(other.update(new_timestamp, new_value).is_ok());

    assert_eq!(other.value, new_value);
    assert_eq!(other.value(), &new_value);
    assert_eq!(other.last_change(), new_timestamp);
    assert!(fluent < other);

    assert!(other.update(1339, new_value).is_ok());
    assert_eq!(other.last_change(), new_timestamp);
    assert!(fluent < other);
  }

  #[test]
  fn boolean_fluent_test() {
    assert_impl_all!(Fluent<bool>: AnyFluent);

    let name = "numeric_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = true;

    let fluent = Fluent::new(name, keys, timestamp, value);

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

    let mut other = Fluent::new(name, keys, timestamp, value);
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
  }
}
