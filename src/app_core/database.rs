// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::fluent::ValueType;

use eyre::Result;
use serde::Deserialize;
use std::time::Duration;
use tokio::time;
use tokio_postgres as tp;
use tracing::{error, info};


#[derive(Debug, Clone, Deserialize)]
/// "Singleton" used to establish database connections from one single place
/// and spawn database request handlers.
pub struct Database {
  host:     String,
  user:     String,
  password: String,
  dbname:   String,
  #[serde(skip)]
  template: String,
}

impl Database {
  /// Establishes a connection to the database and returns a database client
  /// handle on success.
  pub async fn connect(&self) -> Result<tp::Client> {
    let params_str =
      format!("host={} user={} password={} dbname={}",
              &self.host, &self.user, &self.password, &self.dbname);

    let (client, connection) = tp::connect(&params_str, tp::NoTls).await?;

    // task awaits database connection, traces on error
    tokio::spawn(async move {
      if let Err(e) = connection.await {
        error!("connection error: {}", e);
      }
    });

    Ok(client)
  }

  /// Set the query template for this instance.
  pub fn with_template_option(mut self, template: Option<&str>) -> Self {
    if let Some(template) = template {
      self.template = template.to_string();
    }
    self
  }

  /// Query the database using the statement template. Queries must return
  /// exactly one value in exactly one row for this to return `Ok(T)`.
  pub async fn query<T>(&self,
                        params: &[&(dyn tp::types::ToSql + Sync)])
                        -> Option<T>
    where T: ValueType + for<'a> tp::types::FromSql<'a>
  {
    if self.template.is_empty() {
      return None;
    }

    let client = self.connect().await.ok()?;
    let query_future = client.query_one(&self.template, params);
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

    row.try_get::<usize, T>(0).ok()
  }
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::Database;

  use indoc::indoc;
  use pretty_assertions::assert_eq;


  #[test]
  fn database_test() {
    let host = String::from("morpheus");
    let user = String::from("neo");
    let password = String::from("trinity");
    let dbname = String::from("nebukadnezar");
    let template = String::from("select * from bla");

    let dbc = Database { host:     host.clone(),
                         user:     user.clone(),
                         password: password.clone(),
                         dbname:   dbname.clone(),
                         template: template.clone(), };

    // "dumb" tests
    assert_eq!(dbc.host, host);
    assert_eq!(dbc.user, user);
    assert_eq!(dbc.password, password);
    assert_eq!(dbc.dbname, dbname);
    assert_eq!(dbc.template, template);

    let match_str = indoc! {r#"
      Database {
          host: "morpheus",
          user: "neo",
          password: "trinity",
          dbname: "nebukadnezar",
          template: "select * from bla",
      }"#};

    assert_eq!(format!("{:#?}", dbc), match_str);

    // ideally the methods connection to the database could be tested here as
    // well, for that to happen, we'd need either a mock database or a
    // container of some sort to run with the tests. which i don't have time or
    // patience to set up right now.
  }
}
