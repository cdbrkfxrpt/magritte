// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Fluent, Key, Timestamp};


#[derive(Clone, Debug, PartialEq)]
/// Enables sending `Fluent`s through channels.
pub enum AnyFluent {
  Textual(Fluent<String>),
  Integer(Fluent<i64>),
  FloatPt(Fluent<f64>),
  Boolean(Fluent<bool>),
}

impl AnyFluent {
  pub fn new<ValueType: Selector>(name: &str,
                                  keys: &[Key],
                                  timestamp: Timestamp,
                                  value: ValueType)
                                  -> Self {
    ValueType::select(name, keys, timestamp, value)
  }
}

pub trait Selector {
  fn select(name: &str,
            keys: &[Key],
            timestamp: Timestamp,
            value: Self)
            -> AnyFluent;
}

impl Selector for String {
  fn select(name: &str,
            keys: &[Key],
            timestamp: Timestamp,
            value: Self)
            -> AnyFluent {
    AnyFluent::Textual(Fluent::new(name, keys, timestamp, value))
  }
}

impl Selector for i64 {
  fn select(name: &str,
            keys: &[Key],
            timestamp: Timestamp,
            value: Self)
            -> AnyFluent {
    AnyFluent::Integer(Fluent::new(name, keys, timestamp, value))
  }
}

impl Selector for f64 {
  fn select(name: &str,
            keys: &[Key],
            timestamp: Timestamp,
            value: Self)
            -> AnyFluent {
    AnyFluent::FloatPt(Fluent::new(name, keys, timestamp, value))
  }
}

impl Selector for bool {
  fn select(name: &str,
            keys: &[Key],
            timestamp: Timestamp,
            value: Self)
            -> AnyFluent {
    AnyFluent::Boolean(Fluent::new(name, keys, timestamp, value))
  }
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::{AnyFluent, Fluent};

  use pretty_assertions::assert_eq;


  #[test]
  fn textual_fluent_test() {
    let name = "textual_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = String::from("running");

    let any_fluent = AnyFluent::new(name, keys, timestamp, value.clone());

    assert!(matches!(any_fluent, AnyFluent::Textual(..)));

    let AnyFluent::Textual(extracted) = any_fluent.clone() else { panic!() };
    let fluent = Fluent::new(name, keys, timestamp, value.clone());

    assert_eq!(extracted, fluent);

    let dbg_print = format!("{:?}", any_fluent);
    assert_eq!(&dbg_print,
               r#"Textual(Fluent { name: "textual_fluent", keys: [23, 42], timestamp: 1337, value: "running", last_change: 1337 })"#);
  }

  #[test]
  fn integer_fluent_test() {
    let name = "integer_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = 3;

    let any_fluent = AnyFluent::new(name, keys, timestamp, value);

    assert!(matches!(any_fluent, AnyFluent::Integer(..)));

    let AnyFluent::Integer(extracted) = any_fluent else { panic!() };
    let fluent = Fluent::new(name, keys, timestamp, value);

    assert_eq!(extracted, fluent);
  }

  #[test]
  fn floatpt_fluent_test() {
    let name = "floatpt_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = 3.14159;

    let any_fluent = AnyFluent::new(name, keys, timestamp, value);

    assert!(matches!(any_fluent, AnyFluent::FloatPt(..)));

    let AnyFluent::FloatPt(extracted) = any_fluent else { panic!() };
    let fluent = Fluent::new(name, keys, timestamp, value);

    assert_eq!(extracted, fluent);
  }

  #[test]
  fn boolean_fluent_test() {
    let name = "boolean_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = true;

    let any_fluent = AnyFluent::new(name, keys, timestamp, value);

    assert!(matches!(any_fluent, AnyFluent::Boolean(..)));

    let AnyFluent::Boolean(extracted) = any_fluent else { panic!() };
    let fluent = Fluent::new(name, keys, timestamp, value);

    assert_eq!(extracted, fluent);
  }
}
