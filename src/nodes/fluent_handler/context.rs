// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::fluent::ValueType;

use derivative::Derivative;
use eyre::{bail, ensure, Result};
use std::sync::Arc;
use tokio_postgres::{types::{FromSql, ToSql},
                     Client,
                     Statement};


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
                             -> Result<T>
    where T: ValueType + for<'a> FromSql<'a>
  {
    let row = self.client.query_one(&self.statement, params).await?;
    ensure!(row.len() == 1,
            "database query must return exactly one value");

    Ok(row.get::<usize, T>(0))
  }
}


#[derive(Derivative)]
#[derivative(Debug)]
/// Container for run time parameters used in user definitions of the
/// [`FluentHandler`](super::FluentHandler) as it runs.
pub struct Context {
  #[derivative(Debug = "ignore")]
  database: Option<DatabaseContext>,
}

impl Context {
  /// Use inputs to set up new [`Context`] with e.g. a [`DatabaseContext`].
  pub async fn new(client: Client, query: Option<&str>) -> Result<Self> {
    let database = match query {
      Some(query) => Some(DatabaseContext::new(client, query).await?),
      None => None,
    };

    Ok(Self { database })
  }

  /// Query the contained database connection, if available, with the statement
  /// template generated into a query using the given params.
  pub async fn database_query<T>(&self,
                                 params: &[&(dyn ToSql + Sync)])
                                 -> Result<T>
    where T: ValueType + for<'a> FromSql<'a>
  {
    match &self.database {
      Some(database) => Ok(database.query_with::<T>(params).await?),
      None => bail!("Context has no database information"),
    }
  }

  pub fn arc_ptr(self) -> Arc<Self> {
    Arc::new(self)
  }
}
