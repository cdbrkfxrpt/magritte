// Copyright 2022 bmc::labs Sagl.
//
// Authors:
//   Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

//! Components implementing static fluent behavior lives here.
//!
//! This includes the [`Fluent`] struct itself, which implements the
//! characteristics of a fluent directly using a variable of generic type as
//! the fluent value type. It also includes the [`AnyFluent`] enum, which wraps
//! [`Fluent`] and provides some convenience functionality to deal with
//! [`Fluent`]s, and the [`FluentValue`] trait which helps [`AnyFluent`] and
//! the rest of the application deal with the generic value type of [`Fluent`].
//! Finally, [`EvalFn`] is a wrapper for a user defined closure to evaluate the
//! value of fluents.

mod any_fluent;
mod eval_fn;
mod fluent;
mod fluent_value;

pub use any_fluent::AnyFluent;
pub use eval_fn::EvalFn;
pub use fluent::Fluent;
pub use fluent_value::FluentValue;


/// Type alias for key, i.e. sub-stream identifier, type.
pub type Key = usize;
/// Type alias for timestamp type.
pub type Timestamp = usize;
