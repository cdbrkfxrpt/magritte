// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::config::DatabaseParams;

use eyre::Result;
use indoc::indoc;
use tokio_postgres as tp;
use tracing::{error, info};


#[derive(Debug)]
pub struct DatabaseHandler {
  params_str: String,
}

impl DatabaseHandler {
  pub fn new(params: DatabaseParams) -> Self {
    let params_str =
      format!("host={} user={} password={} dbname={}",
              params.host, params.user, params.password, params.dbname);
    Self { params_str }
  }

  pub async fn connect(&self) -> Result<tp::Client> {
    let (client, connection) = tp::connect(&self.params_str, tp::NoTls).await?;
    info!("database connection successful");

    // task awaits database connection, traces on error
    tokio::spawn(async move {
      if let Err(e) = connection.await {
        error!("connection error: {}", e);
      }
    });

    Ok(client)
  }

  pub async fn prepare_run(&self) -> Result<()> {
    let client = self.connect().await?;

    let sql_raw = indoc! {r#"
      -- dropping and recreating magritte schema
      drop schema if exists magritte cascade;
      create schema magritte;

      -- creating event stream table
      create table magritte.event_stream (
        id          serial,
        fluent_name text,
        keys        integer[],
        timestamp   bigint,
        value       bool,
        lastChanged bigint
      );

      -- transforming coastline to the correct coordinate system:
      -- this is very costly but can be done only once on startup
      select gid, shape_leng, ST_Transform(geom, 3857) as geom
      into   magritte.europe_coastline
      from   geographic_features.europe_coastline;
    "#};

    info!("executing run preparation SQL:\n\n{}", sql_raw);
    client.batch_execute(sql_raw).await?;

    Ok(())
  }
}
