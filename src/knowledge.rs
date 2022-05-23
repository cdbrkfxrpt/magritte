// Copyright 2022 Florian Eich <florian.eich@gmail.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use indoc::indoc;
use std::collections::HashMap;
use tokio_postgres::{Client, Statement};


pub async fn build_functions_index(client: &Client)
                                   -> HashMap<String, Statement> {
  let mut statements = HashMap::new();

  let raw_statements = [("distance_from_coastline", DISTANCE_FROM_COASTLINE)];

  for raw_statement in raw_statements {
    let statement = match client.prepare(raw_statement.1).await {
      Ok(statement) => statement,
      Err(_) => panic!("could not prepare statement '{}'", raw_statement.0),
    };

    statements.insert(raw_statement.0.to_string(), statement);
  }

  statements
}

const DISTANCE_FROM_COASTLINE: &str = indoc! {r#"
  select ST_Distance(
    ST_Transform('SRID=4326;POINT($1 $2)'::geometry, 3857),
    ST_Transform(geom, 3857)
  )
  from
    "geographic_features"."europe_coastline"
  limit 1
"#};
