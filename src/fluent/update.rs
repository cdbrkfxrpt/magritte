// Copyright 2022 bmc::labs Sagl.
//
// Authors:
//   Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use super::Timestamp;

use eyre::Result;


pub trait Update {
  type ValueType;

  fn update(&mut self,
            timestamp: Timestamp,
            value: Self::ValueType)
            -> Result<()>;
}
