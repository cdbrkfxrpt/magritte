// Copyright 2022 bmc::labs Sagl.
//
// Authors:
//   Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

mod any_fluent;
mod fluent;
mod fluent_handler;

pub use any_fluent::AnyFluent;
pub use fluent::Fluent;
pub use fluent_handler::FluentHandler;


/// Type alias for key, i.e. sub-stream identifier, type.
pub type Key = usize;
/// Type alias for timestamp type.
pub type Timestamp = usize;
