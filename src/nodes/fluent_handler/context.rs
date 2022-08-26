// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::fluent::{Fluent, ValueType};

use derivative::Derivative;
use eyre::Result;
use std::{sync::Arc, time::Duration};
use tokio::time;
use tokio_postgres::{types::{FromSql, ToSql},
                     Client,
                     Statement};
use tracing::info;


#[derive(Derivative)]
#[derivative(Debug)]
/// Helper struct used by [`Context`] to bundle database related elements.
struct DatabaseContext {
  client:    Client,
  #[derivative(Debug = "ignore")]
  statement: Statement,
}

impl DatabaseContext {
  /// Prepares the database query template and sets up the [`DatabaseContext`].
  pub async fn new(client: Client, query: &str) -> Result<Self> {
    let statement = client.prepare(query).await?;
    Ok(Self { client, statement })
  }

  /// Query the database using the prepared statement template. Queries must
  /// return exactly one value in exactly one row for this to return `Ok(T)`.
  pub async fn query_with<T>(&self,
                             params: &[&(dyn ToSql + Sync)])
                             -> Option<T>
    where T: ValueType + for<'a> FromSql<'a>
  {
    let query_future = self.client.query_one(&self.statement, params);
    let row =
      match time::timeout(Duration::from_millis(150), query_future).await {
        Ok(query_result) => query_result.ok()?,
        Err(_) => {
          info!("database query timed out");
          return None;
        }
      };

    if row.len() != 1 {
      info!("database query returned more than one value");
      return None;
    }

    Some(row.get::<usize, T>(0))
  }
}


#[derive(Derivative)]
#[derivative(Debug)]
/// Container for run time parameters used in user definitions of the
/// [`FluentHandler`](super::FluentHandler) as it runs.
pub struct Context {
  #[derivative(Debug = "ignore")]
  database:  Option<DatabaseContext>,
  reference: Vec<Fluent>,
}

impl Context {
  /// Use inputs to set up new [`Context`] with e.g. a [`DatabaseContext`].
  pub async fn new(client: Client, query: Option<&str>) -> Result<Self> {
    let database = match query {
      Some(query) => Some(DatabaseContext::new(client, query).await?),
      None => None,
    };
    let reference = Vec::new();

    Ok(Self { database,
              reference })
  }

  /// Query the contained database connection, if available, with the statement
  /// template generated into a query using the given params.
  pub async fn database_query<T>(&self,
                                 params: &[&(dyn ToSql + Sync)])
                                 -> Option<T>
    where T: ValueType + for<'a> FromSql<'a>
  {
    match &self.database {
      Some(database) => Some(database.query_with::<T>(params).await?),
      None => None,
    }
  }

  pub fn arc_ptr(self) -> Arc<Self> {
    Arc::new(self)
  }
}
