// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::config::Config;

use eyre::Result;
use indoc::indoc;
use tokio_postgres as tp;
use tracing::error;


pub async fn run_prepare(config: &Config) -> Result<()> {
  let dbparams = format!("host={} user={} password={} dbname={}",
                         config.database_credentials.host,
                         config.database_credentials.user,
                         config.database_credentials.password,
                         config.database_credentials.dbname);

  let (dbclient, connection) = tp::connect(&dbparams, tp::NoTls).await?;

  tokio::spawn(async move {
    if let Err(e) = connection.await {
      error!("connection error: {}", e);
    }
  });

  dbclient.batch_execute(indoc! {r#"
    drop schema if exists magritte cascade;
    create schema magritte;
    create table magritte.results (
      source_id   serial,
      timestamp   bigint,
      fluent_name text,
      holds       bool
    );
    select gid, shape_leng, ST_Transform(geom, 3857) as geom
    into   magritte.europe_coastline
    from   geographic_features.europe_coastline;
    "#})
          .await?;

  Ok(())
}
