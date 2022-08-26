// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

//! Components which manage fluents during run time live here.
//!
//! The main component here is the [`Handler`], which implements
//! [`Node`](crate::app_core::Node) and is the element used by the
//! [`AppCore`](crate::app_core::AppCore) to manage fluent progress over time.
//!
//! The component [`HandlerDefinition`] is what allows users of `magritte` to
//! easily input the elements a [`Handler`] requires to run: the name of the
//! fluent, its dependencies (i.e. other fluents it needs to know the value of
//! to progress), an optional PostgreSQL database query statement template and
//! an evaluation function.
//!
//! The evaluation function is given as a wrapped `async` closure - the
//! [`EvalFn`] component is the wrapper for the closure. This is necessary to
//! satisfy trait and lifetime bounds of the closure whilst maintaining an
//! ergonomic way of providing an evaluation function to the user.
//!
//! To understand usage, see the example definitions in the
//! `conf/fluent_handlers.rs` file of the repo.

mod eval_fn;
mod handler;

pub use eval_fn::EvalFn;
pub use handler::{Handler, HandlerDefinition, KeyDependency};
