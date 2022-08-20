// Copyright 2022 bmc::labs Sagl.
//
// Authors:
//   Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

//! Components implementing static fluent behavior live here.
//!
//! This includes the [`InnerFluent`] struct, which implements the
//! characteristics of a fluent directly using a variable of generic type as
//! the fluent value type. It also includes the [`Fluent`] enum, which wraps
//! [`InnerFluent`] and provides some convenience functionality to deal with
//! [`InnerFluent`]s, and the [`ValueType`] trait which helps [`Fluent`] and
//! the rest of the application deal with the generic value type of
//! [`InnerFluent`].

mod fluent;
mod inner_fluent;
mod value_type;

pub use fluent::Fluent;
pub use inner_fluent::InnerFluent;
pub use value_type::ValueType;

// fin re-exports ---------------------------------------------------------- //

/// Type alias for key, i.e. sub-stream identifier, type.
pub type Key = usize;
/// Type alias for timestamp type.
pub type Timestamp = usize;

/// Trait for fluents.
pub trait FluentTrait {
  fn name(&self) -> &str;
  fn keys(&self) -> &[Key];
  fn timestamp(&self) -> Timestamp;
  fn boxed_value(&self) -> Box<dyn ValueType>;
  fn last_change(&self) -> Timestamp;
}
