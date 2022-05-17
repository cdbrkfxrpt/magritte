// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::Fluent;
use crate::types::{Datapoint, FluentResult};


#[derive(Debug)]
pub struct NeutralFluent;

impl Fluent for NeutralFluent {
  fn rule(&self, datapoint: Datapoint) -> Option<FluentResult> {
    Some(FluentResult { datapoint,
                        name: String::from("Neutral Fluent"),
                        description: String::from("forwards datapoint") })
  }
}


// #[derive(Debug)]
// pub struct NearCoast {}

// impl Fluent for NearCoast {
//   fn rule(&self, datapoint: Datapoint)
// }
