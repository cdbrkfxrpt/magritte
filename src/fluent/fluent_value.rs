// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{AnyFluent, Key, Timestamp};

use std::fmt;


/// Helper trait to enable selection of the correct [`AnyFluent`] variant based
/// on the type of the provided value.
pub trait FluentValue: fmt::Debug {
  /// Implement this function for your type, returning an [`AnyFluent`] variant
  /// with a new [`Fluent`] inside.
  ///
  /// Implementations are provided for:
  /// - `String`
  /// - `i64`
  /// - `f64`
  /// - `bool`
  /// - `(f64, f64)`
  fn to_fluent(&self, name: &str, keys: &[Key], ts: Timestamp) -> AnyFluent;
}
