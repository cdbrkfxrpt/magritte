// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

//! Contains the core functionality components of `magritte`.
//!
//! This includes the [`AppCore`] struct, which sets up and runs the
//! application, the [`Broker`] struct, which handles message passing from
//! publishers to subscribers of fluents, the [`Database`] struct and all its
//! related elements used to interact with the PostgreSQL database and finally
//! a number of [`util`] components.

mod app_core;
mod broker;
mod database;
pub mod util;

pub use app_core::AppCore;
pub use broker::Broker;
pub use database::Database;
