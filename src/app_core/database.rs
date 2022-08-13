// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::fluent::AnyFluent;

use eyre::Result;
use serde::Deserialize;
use tokio::{sync::{mpsc, oneshot},
            task::JoinHandle};
use tokio_postgres as tp;
use tracing::{error, info};


pub type RequestTx = mpsc::UnboundedSender<DatabaseRequest>;
pub type RequestRx = mpsc::UnboundedReceiver<DatabaseRequest>;


#[derive(Debug)]
pub struct DatabaseRequest {
  fn_name:     String,
  fn_input:    Vec<AnyFluent>,
  response_tx: oneshot::Sender<AnyFluent>,
}


#[derive(Debug, Deserialize)]
/// "Singleton" used to establish database connections from one single place
/// and spawn database request handlers.
pub struct Database {
  host:       String,
  user:       String,
  password:   String,
  dbname:     String,
  #[serde(skip, default = "mpsc::unbounded_channel")]
  request_ch: (RequestTx, RequestRx),
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

    info!("database connection successful");
    Ok(client)
  }

  pub fn request_tx(&self) -> RequestTx {
    self.request_ch.0.clone()
  }

  pub fn request_handler(&mut self) -> JoinHandle<()> {
    let new_db = Self { host:       self.host.clone(),
                        user:       self.user.clone(),
                        password:   self.password.clone(),
                        dbname:     self.password.clone(),
                        request_ch: mpsc::unbounded_channel(), };

    let database = std::mem::replace(self, new_db);

    tokio::spawn(async move {
      let _database_client =
        database.connect()
                .await
                .expect("unable to establish database connection");

      // retrieve current receiver handle and update for future calls
      let (_, _request_rx) = database.request_ch;

      // listen to request_rx and handle requests
    })
  }
}

// fin --------------------------------------------------------------------- //

#[cfg(test)]
mod tests {
  use super::Database;

  use indoc::indoc;
  use pretty_assertions::assert_eq;
  use tokio::sync::mpsc;
  use wildmatch::WildMatch;


  #[test]
  fn database_test() {
    let host = String::from("morpheus");
    let user = String::from("neo");
    let password = String::from("trinity");
    let dbname = String::from("nebukadnezar");

    let dbc = Database { host:       host.clone(),
                         user:       user.clone(),
                         password:   password.clone(),
                         dbname:     dbname.clone(),
                         request_ch: mpsc::unbounded_channel(), };

    // "dumb" tests
    assert_eq!(dbc.host, host);
    assert_eq!(dbc.user, user);
    assert_eq!(dbc.password, password);
    assert_eq!(dbc.dbname, dbname);

    let match_str = indoc! {r#"
      Database {
          host: "morpheus",
          user: "neo",
          password: "trinity",
          dbname: "nebukadnezar",
          *
      }"#};

    assert!(WildMatch::new(match_str).matches(&format!("{:#?}", dbc)));

    // ideally the methods connection to the database could be tested here as
    // well, for that to happen, we'd need either a mock database or a
    // container of some sort to run with the tests. which i don't have time or
    // patience to set up right now.
  }
}
