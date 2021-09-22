// Copyright 2021 Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

mod dlns {
  tonic::include_proto!("dylonet");
}

mod datapoint;
mod util;

pub use dlns::*;
