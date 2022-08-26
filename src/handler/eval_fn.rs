// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::{app_core::Database,
            fluent::{Fluent, ValueType}};

use futures::future::BoxFuture;
use std::sync::Arc;


/// Helper type for the closure objects stored in the [`EvalFn`] struct.
type FnType<'a> = Arc<dyn (Fn(Vec<Fluent>,
                           Database)
                           -> BoxFuture<'a, Option<Box<dyn ValueType>>>)
                        + Send
                        + Sync>;


/// Wrapper struct for closures which are used to evaluate fluents.
pub struct EvalFn {
  f: FnType<'static>,
}

impl EvalFn {
  /// Constructor function named in this way for better readability in the user
  /// defined code section, i.e. `EvalFn::specify(/* ... /*)` is deemed _more
  /// obvious_ in terms of naming than, say, `EvalFn::new(/* ... */)` would be.
  pub fn specify(f: FnType<'static>) -> Self {
    Self { f }
  }

  pub fn into_inner(self) -> FnType<'static> {
    self.f
  }
}
