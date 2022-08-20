// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::fluent::ValueType;

use derivative::Derivative;
use eyre::{bail, ensure, Result};
use tokio_postgres::{row::Row,
                     types::{FromSql, ToSql},
                     Client,
                     Statement};


#[derive(Derivative)]
#[derivative(Debug)]
struct DatabaseContext {
  pub client:    Client,
  #[derivative(Debug = "ignore")]
  pub statement: Statement,
}

impl DatabaseContext {
  pub async fn query_with(&self,
                          params: &[&(dyn ToSql + Sync)])
                          -> Result<Row> {
    Ok(self.client.query_one(&self.statement, params).await?)
  }
}


#[derive(Derivative)]
#[derivative(Debug)]
pub struct Context {
  #[derivative(Debug = "ignore")]
  database: Option<DatabaseContext>,
}

impl Context {
  pub async fn new(client: Client, query: Option<&str>) -> Result<Self> {
    let database = match query {
      Some(query) => {
        let statement = client.prepare(query).await?;
        Some(DatabaseContext { client, statement })
      }
      None => None,
    };

    Ok(Self { database })
  }

  pub async fn database_query<T>(&self,
                                 values: &[&(dyn ToSql + Sync)])
                                 -> Result<T>
    where T: ValueType + for<'a> FromSql<'a>
  {
    let row = match &self.database {
      Some(database) => database.query_with(values).await?,
      None => bail!("Context has no database information"),
    };
    ensure!(row.len() == 1,
            "database query must return exactly one value");

    Ok(row.get::<usize, T>(0))
  }
}
