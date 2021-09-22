// Copyright 2021 Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::State;


impl From<i32> for State {
  fn from(i: i32) -> Self {
    match i {
      0 => State::Inactive,
      1 => State::Ready,
      2 => State::Results,
      _ => panic!("State can only be converted from values between 0 and 2"),
    }
  }
}
