// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use eyre::Result;
use serde::Deserialize;
use tokio_postgres as tp;
use tracing::{error, info};


#[derive(Debug, Deserialize)]
/// "Singleton" used to establish database connections from one single place.
pub struct DatabaseConnector {
  host:     String,
  user:     String,
  password: String,
  dbname:   String,
}

impl DatabaseConnector {
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

    info!("database connection successful");
    Ok(client)
  }
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::DatabaseConnector;

  use indoc::indoc;
  use pretty_assertions::assert_eq;


  #[test]
  fn database_connector_test() {
    let host = String::from("morpheus");
    let user = String::from("neo");
    let password = String::from("trinity");
    let dbname = String::from("nebukadnezar");

    let dbc = DatabaseConnector { host:     host.clone(),
                                  user:     user.clone(),
                                  password: password.clone(),
                                  dbname:   dbname.clone(), };

    // "dumb" tests
    assert_eq!(dbc.host, host);
    assert_eq!(dbc.user, user);
    assert_eq!(dbc.password, password);
    assert_eq!(dbc.dbname, dbname);

    let dbg_str = indoc! {r#"
      DatabaseConnector {
          host: "morpheus",
          user: "neo",
          password: "trinity",
          dbname: "nebukadnezar",
      }"#};

    assert_eq!(format!("{:#?}", dbc), dbg_str);

    // ideally the methods connection to the database could be tested here as
    // well, for that to happen, we'd need either a mock database or a
    // container of some sort to run with the tests. which i don't have time or
    // patience to set up right now.
  }
}
