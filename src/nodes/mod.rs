// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

//! [`Node`]s connect to the [`Broker`](crate::app_core::Broker) to publish and
//! subscribe to [`Fluent`](crate::fluent::Fluent)s.
//!
//! Contains all the fun stuff required to implement nodes and send data
//! around between them.

mod fluent_handler;
mod node;
mod sink;
mod source;

pub use fluent_handler::FluentHandler;
pub use node::{Node, NodeRx, NodeTx};
pub use sink::Sink;
pub use source::Source;
