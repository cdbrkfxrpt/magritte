// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{FluentTrait, InnerFluent, Key, Timestamp, ValueType};


#[derive(Clone, Debug, PartialEq)]
/// Enables sending [`Fluent`]s through channels.
pub enum Fluent {
  Textual(InnerFluent<String>),
  Integer(InnerFluent<i64>),
  FloatPt(InnerFluent<f64>),
  Boolean(InnerFluent<bool>),
  PlanePt(InnerFluent<(f64, f64)>),
}

impl Fluent {
  /// Allows the creation of [`Fluent`] objects using any value of a type that
  /// implements the [`ValueType`] trait, which is a helper trait.
  pub fn new(name: &str,
             keys: &[Key],
             timestamp: Timestamp,
             value: Box<dyn ValueType>)
             -> Self {
    value.to_fluent(name, keys, timestamp)
  }

  /// Returns the inner value of the [`Fluent`].
  pub fn value<T: ValueType>(&self) -> T {
    let boxed_value = match self {
      Self::Textual(fluent) => fluent.boxed_value(),
      Self::Integer(fluent) => fluent.boxed_value(),
      Self::FloatPt(fluent) => fluent.boxed_value(),
      Self::Boolean(fluent) => fluent.boxed_value(),
      Self::PlanePt(fluent) => fluent.boxed_value(),
    };

    // unwrap is safe due to trait bounds on T
    Box::into_inner(boxed_value.downcast::<T>().unwrap())
  }
}

impl FluentTrait for Fluent {
  /// Helper function to get fluent name.
  fn name(&self) -> &str {
    match self {
      Self::Textual(fluent) => fluent.name(),
      Self::Integer(fluent) => fluent.name(),
      Self::FloatPt(fluent) => fluent.name(),
      Self::Boolean(fluent) => fluent.name(),
      Self::PlanePt(fluent) => fluent.name(),
    }
  }

  /// Helper function to get fluent keys.
  fn keys(&self) -> &[Key] {
    match self {
      Self::Textual(fluent) => fluent.keys(),
      Self::Integer(fluent) => fluent.keys(),
      Self::FloatPt(fluent) => fluent.keys(),
      Self::Boolean(fluent) => fluent.keys(),
      Self::PlanePt(fluent) => fluent.keys(),
    }
  }

  /// Helper function to get fluent timestamp.
  fn timestamp(&self) -> Timestamp {
    match self {
      Self::Textual(fluent) => fluent.timestamp(),
      Self::Integer(fluent) => fluent.timestamp(),
      Self::FloatPt(fluent) => fluent.timestamp(),
      Self::Boolean(fluent) => fluent.timestamp(),
      Self::PlanePt(fluent) => fluent.timestamp(),
    }
  }

  fn boxed_value(&self) -> Box<dyn ValueType> {
    match self {
      Self::Textual(fluent) => fluent.boxed_value(),
      Self::Integer(fluent) => fluent.boxed_value(),
      Self::FloatPt(fluent) => fluent.boxed_value(),
      Self::Boolean(fluent) => fluent.boxed_value(),
      Self::PlanePt(fluent) => fluent.boxed_value(),
    }
  }

  /// Helper function to get last change of fluent.
  fn last_change(&self) -> Timestamp {
    match self {
      Self::Textual(fluent) => fluent.last_change(),
      Self::Integer(fluent) => fluent.last_change(),
      Self::FloatPt(fluent) => fluent.last_change(),
      Self::Boolean(fluent) => fluent.last_change(),
      Self::PlanePt(fluent) => fluent.last_change(),
    }
  }
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::{Fluent, InnerFluent};

  use pretty_assertions::assert_eq;


  #[test]
  fn textual_fluent_test() {
    let name = "textual_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = String::from("running");

    let any_fluent =
      Fluent::new(name, keys, timestamp, Box::new(value.clone()));

    assert!(matches!(any_fluent, Fluent::Textual(..)));
    assert_eq!(any_fluent.name(), name);

    let Fluent::Textual(extracted) = any_fluent.clone() else { panic!() };
    let fluent = InnerFluent::new(name, keys, timestamp, &value);

    assert_eq!(extracted, fluent);

    let dbg_print = format!("{:?}", any_fluent);
    assert_eq!(&dbg_print,
               r#"Textual(InnerFluent { name: "textual_fluent", keys: [23, 42], timestamp: 1337, value: "running", last_change: 1337 })"#);
  }

  #[test]
  fn integer_fluent_test() {
    let name = "integer_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = 3;

    let any_fluent = Fluent::new(name, keys, timestamp, Box::new(value));

    assert!(matches!(any_fluent, Fluent::Integer(..)));
    assert_eq!(any_fluent.name(), name);

    let Fluent::Integer(extracted) = any_fluent.clone() else { panic!() };
    let fluent = InnerFluent::new(name, keys, timestamp, &value);

    assert_eq!(extracted, fluent);

    let dbg_print = format!("{:?}", any_fluent);
    assert_eq!(&dbg_print,
               r#"Integer(InnerFluent { name: "integer_fluent", keys: [23, 42], timestamp: 1337, value: 3, last_change: 1337 })"#);
  }

  #[test]
  fn floatpt_fluent_test() {
    let name = "floatpt_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = 3.14159;

    let any_fluent = Fluent::new(name, keys, timestamp, Box::new(value));

    assert!(matches!(any_fluent, Fluent::FloatPt(..)));
    assert_eq!(any_fluent.name(), name);

    let Fluent::FloatPt(extracted) = any_fluent.clone() else { panic!() };
    let fluent = InnerFluent::new(name, keys, timestamp, &value);

    assert_eq!(extracted, fluent);

    let dbg_print = format!("{:?}", any_fluent);
    assert_eq!(&dbg_print,
               r#"FloatPt(InnerFluent { name: "floatpt_fluent", keys: [23, 42], timestamp: 1337, value: 3.14159, last_change: 1337 })"#);
  }

  #[test]
  fn boolean_fluent_test() {
    let name = "boolean_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = true;

    let any_fluent = Fluent::new(name, keys, timestamp, Box::new(value));

    assert!(matches!(any_fluent, Fluent::Boolean(..)));
    assert_eq!(any_fluent.name(), name);

    let Fluent::Boolean(extracted) = any_fluent.clone() else { panic!() };
    let fluent = InnerFluent::new(name, keys, timestamp, &value);

    assert_eq!(extracted, fluent);

    let dbg_print = format!("{:?}", any_fluent);
    assert_eq!(&dbg_print,
               r#"Boolean(InnerFluent { name: "boolean_fluent", keys: [23, 42], timestamp: 1337, value: true, last_change: 1337 })"#);
  }

  #[test]
  fn planept_fluent_test() {
    let name = "planept_fluent";
    let keys = &[23, 42];
    let timestamp = 1337;
    let value = (3.14159, 2.71828);

    let any_fluent = Fluent::new(name, keys, timestamp, Box::new(value));

    assert!(matches!(any_fluent, Fluent::PlanePt(..)));
    assert_eq!(any_fluent.name(), name);

    let Fluent::PlanePt(extracted) = any_fluent.clone() else { panic!() };
    let fluent = InnerFluent::new(name, keys, timestamp, &value);

    assert_eq!(extracted, fluent);

    let dbg_print = format!("{:?}", any_fluent);
    assert_eq!(&dbg_print,
               r#"PlanePt(InnerFluent { name: "planept_fluent", keys: [23, 42], timestamp: 1337, value: (3.14159, 2.71828), last_change: 1337 })"#);
  }
}
