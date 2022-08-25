// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{Fluent, InnerFluent, Key, Timestamp};

use downcast_rs::{impl_downcast, DowncastSync};
use std::{fmt, time::Instant};


/// Helper trait to enable selection of the correct [`Fluent`] variant based
/// on the type of the provided value.
pub trait ValueType: fmt::Debug + DowncastSync {
  /// Implement this function for your type, returning an [`Fluent`] variant
  /// with a new [`Fluent`](super::Fluent) inside.
  ///
  /// Implementations are provided for:
  /// - `String`
  /// - `i32`
  /// - `i64`
  /// - `f64`
  /// - `bool`
  /// - `(f64, f64)`
  fn to_fluent(&self, name: &str, keys: &[Key], ts: Timestamp) -> Fluent;
}

impl_downcast!(sync ValueType);

impl ValueType for String {
  fn to_fluent(&self, name: &str, keys: &[Key], ts: Timestamp) -> Fluent {
    Fluent::Textual(InnerFluent::new(name, keys, ts, self.to_owned()))
  }
}

impl ValueType for i32 {
  fn to_fluent(&self, name: &str, keys: &[Key], ts: Timestamp) -> Fluent {
    Fluent::Integer(InnerFluent::new(name, keys, ts, self.to_owned()))
  }
}

impl ValueType for i64 {
  fn to_fluent(&self, name: &str, keys: &[Key], ts: Timestamp) -> Fluent {
    Fluent::LongInt(InnerFluent::new(name, keys, ts, self.to_owned()))
  }
}

impl ValueType for f64 {
  fn to_fluent(&self, name: &str, keys: &[Key], ts: Timestamp) -> Fluent {
    Fluent::FloatPt(InnerFluent::new(name, keys, ts, self.to_owned()))
  }
}

impl ValueType for bool {
  fn to_fluent(&self, name: &str, keys: &[Key], ts: Timestamp) -> Fluent {
    Fluent::Boolean(InnerFluent::new(name, keys, ts, self.to_owned()))
  }
}

impl ValueType for (f64, f64) {
  fn to_fluent(&self, name: &str, keys: &[Key], ts: Timestamp) -> Fluent {
    Fluent::PlanePt(InnerFluent::new(name, keys, ts, self.to_owned()))
  }
}

impl ValueType for Instant {
  fn to_fluent(&self, name: &str, keys: &[Key], ts: Timestamp) -> Fluent {
    Fluent::Instant(InnerFluent::new(name, keys, ts, self.to_owned()))
  }
}
