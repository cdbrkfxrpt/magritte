// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{AnyFluent, FluentValue};


/// Helper type for the closure objects stored in the [`EvalFn`] struct.
type FnType = Box<dyn (Fn(Vec<AnyFluent>) -> Box<dyn FluentValue>) + Send>;


/// Wrapper struct for closures which are used to evaluate fluents.
pub struct EvalFn {
  f: FnType,
}

impl EvalFn {
  /// Constructor function named in this way for better readability in the user
  /// defined code section, i.e. `EvalFn::specify(/* ... /*)` is deemed _more
  /// obvious_ in terms of naming than, say, `EvalFn::new(/* ... */)` would be.
  pub fn specify(f: FnType) -> Self {
    Self { f }
  }

  /// Function used by the [`FluentHandler`](crate::nodes::FluentHandler) to
  /// execute the evaluation function with given inputs.
  pub fn evaluate_with(&self,
                       fluents: Vec<AnyFluent>)
                       -> Box<dyn FluentValue> {
    (self.f)(fluents)
  }
}
