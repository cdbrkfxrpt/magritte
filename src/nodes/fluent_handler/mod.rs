// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

//! Components which manage fluents during run time live here.
//!
//! The main component here is the [`FluentHandler`], which implements
//! [`Node`](super::Node) and is the element used by the
//! [`AppCore`](crate::app_core::AppCore) to manage fluent progress over time.
//!
//! The component [`FluentHandlerDefinition`] is what allows users of
//! `magritte` to easily input the elements a [`FluentHandler`] requires to
//! run: the name of the fluent, its dependencies (i.e. other fluents it needs
//! to know the value of to progress), an optional PostgreSQL database query
//! statement template and an evaluation function.
//!
//! The evaluation function is given as a wrapped `async` closure - the
//! [`EvalFn`] component is the wrapper for the closure. This is necessary to
//! satisfy trait and lifetime bounds of the closure whilst maintaining an
//! ergonomic way of providing an evaluation function to the user.
//!
//! Finally, [`Context`] is a helper struct containing run time properties
//! required by the user in the evaluation function. Crucially, this includes a
//! prepared database connection (if a template query statement was provided)
//! which requires only the input of query parameters. To understand usage, see
//! the example definitions in the `conf/fluent_handlers.rs` file of the repo.

mod context;
mod eval_fn;
mod handler;

pub use context::Context;
pub use eval_fn::EvalFn;
pub use handler::{FluentHandler, FluentHandlerDefinition, KeyDependency};
