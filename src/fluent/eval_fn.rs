// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::{AnyFluent, FluentValue};


type FnType = Box<dyn (Fn(Vec<AnyFluent>) -> Box<dyn FluentValue>) + Send>;


pub struct EvalFn {
  f: FnType,
}

impl EvalFn {
  pub fn specify(f: FnType) -> Self {
    Self { f }
  }

  pub fn evaluate_with(&self,
                       fluents: Vec<AnyFluent>)
                       -> Box<dyn FluentValue> {
    (self.f)(fluents)
  }
}
