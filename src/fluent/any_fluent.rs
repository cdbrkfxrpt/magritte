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
  PlanePt(Fluent<(f64, f64)>),
}

impl AnyFluent {
  /// Allows the creation of `AnyFluent` objects using any value of a type that
  /// implements the `Selector` trait, which is a helper trait.
  pub fn new<ValueType: Selector>(name: &str,
                                  keys: &[Key],
                                  timestamp: Timestamp,
                                  value: ValueType)
                                  -> Self {
    ValueType::select(name, keys, timestamp, value)
  }

  /// Helper function to get fluent name.
  pub fn name(&self) -> &str {
    match self {
      Self::Textual(fluent) => fluent.name(),
      Self::Integer(fluent) => fluent.name(),
      Self::FloatPt(fluent) => fluent.name(),
      Self::Boolean(fluent) => fluent.name(),
      Self::PlanePt(fluent) => fluent.name(),
    }
  }
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

impl Selector for (f64, f64) {
  fn select(name: &str,
            keys: &[Key],
            timestamp: Timestamp,
            value: Self)
            -> AnyFluent {
    AnyFluent::PlanePt(Fluent::new(name, keys, timestamp, value))
  }
}

/// Helper trait to enable selection of the correct `AnyFluent` variant based
/// on the type of the provided value.
pub trait Selector {
  /// Implement this function for your type, returning an `AnyFluent` variant
  /// with a new `Fluent` inside. Implementations are provided for: `String`,
  /// `i64`, `f64`, `bool`.
  fn select(name: &str,
            keys: &[Key],
            timestamp: Timestamp,
            value: Self)
            -> AnyFluent;
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
    assert_eq!(any_fluent.name(), name);

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
    assert_eq!(any_fluent.name(), name);

    let AnyFluent::Integer(extracted) = any_fluent.clone() else { panic!() };
    let fluent = Fluent::new(name, keys, timestamp, value);

    assert_eq!(extracted, fluent);

    let dbg_print = format!("{:?}", any_fluent);
    assert_eq!(&dbg_print,
               r#"Integer(Fluent { name: "integer_fluent", keys: [23, 42], timestamp: 1337, value: 3, last_change: 1337 })"#);
  }

  #[test]
  fn floatpt_fluent_test() {
    let name = "floatpt_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = 3.14159;

    let any_fluent = AnyFluent::new(name, keys, timestamp, value);

    assert!(matches!(any_fluent, AnyFluent::FloatPt(..)));
    assert_eq!(any_fluent.name(), name);

    let AnyFluent::FloatPt(extracted) = any_fluent.clone() else { panic!() };
    let fluent = Fluent::new(name, keys, timestamp, value);

    assert_eq!(extracted, fluent);

    let dbg_print = format!("{:?}", any_fluent);
    assert_eq!(&dbg_print,
               r#"FloatPt(Fluent { name: "floatpt_fluent", keys: [23, 42], timestamp: 1337, value: 3.14159, last_change: 1337 })"#);
  }

  #[test]
  fn boolean_fluent_test() {
    let name = "boolean_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = true;

    let any_fluent = AnyFluent::new(name, keys, timestamp, value);

    assert!(matches!(any_fluent, AnyFluent::Boolean(..)));
    assert_eq!(any_fluent.name(), name);

    let AnyFluent::Boolean(extracted) = any_fluent.clone() else { panic!() };
    let fluent = Fluent::new(name, keys, timestamp, value);

    assert_eq!(extracted, fluent);

    let dbg_print = format!("{:?}", any_fluent);
    assert_eq!(&dbg_print,
               r#"Boolean(Fluent { name: "boolean_fluent", keys: [23, 42], timestamp: 1337, value: true, last_change: 1337 })"#);
  }

  #[test]
  fn planept_fluent_test() {
    let name = "planept_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = (3.14159, 2.71828);

    let any_fluent = AnyFluent::new(name, keys, timestamp, value);

    assert!(matches!(any_fluent, AnyFluent::PlanePt(..)));
    assert_eq!(any_fluent.name(), name);

    let AnyFluent::PlanePt(extracted) = any_fluent.clone() else { panic!() };
    let fluent = Fluent::new(name, keys, timestamp, value);

    assert_eq!(extracted, fluent);

    let dbg_print = format!("{:?}", any_fluent);
    assert_eq!(&dbg_print,
               r#"PlanePt(Fluent { name: "planept_fluent", keys: [23, 42], timestamp: 1337, value: (3.14159, 2.71828), last_change: 1337 })"#);
  }
}
