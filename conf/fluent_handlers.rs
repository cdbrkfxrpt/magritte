[
  FluentHandlerDefinition {
    fluent_name: "high_speed_near_coast",
    dependencies: &["high_speed", "near_coast"],
    database_query: None,
    eval_fn: EvalFn::specify(Box::new(
      |fluents, _| async move {
        let high_speed = fluents.get(0)?.value::<bool>();
        let near_coast = fluents.get(1)?.value::<bool>();

        usr::return_value(high_speed && near_coast)
      }.boxed()
    ))
  },
  FluentHandlerDefinition {
    fluent_name: "high_speed",
    dependencies: &["speed"],
    database_query: None,
    eval_fn: EvalFn::specify(Box::new(
      |fluents, _| async move {
        let speed_fluent = fluents.get(0)?;

        // optional check - fluents arrive in order given by dependency list
        if speed_fluent.name() != "speed"
           || !matches!(speed_fluent, Fluent::FloatPt(_)) {
          panic!();
        }

        let speed = speed_fluent.value::<f64>();
        usr::return_value(speed >= 5.0)
      }.boxed()
    ))
  },
  FluentHandlerDefinition {
    fluent_name: "near_coast",
    dependencies: &["distance_from_coast"],
    database_query: None,
    eval_fn: EvalFn::specify(Box::new(
      |fluents, _| async move {
        let distance_from_coast = fluents.get(0)?.value::<f64>();
        usr::return_value(distance_from_coast <= 300.0)
      }.boxed()
    ))
  },
  FluentHandlerDefinition {
    fluent_name: "distance_from_coast",
    dependencies: &["lon", "lat"],
    database_query: Some(indoc! {r#"
      -- requires two input values: [lon, lat]
      select ST_Distance(
        ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857),
        geom
      ) as distance
      from magritte.europe_coastline
      limit 1
    "#}),
    eval_fn: EvalFn::specify(Box::new(
      |fluents, context| async move {
        let lon = fluents.get(0)?.value::<f64>();
        let lat = fluents.get(1)?.value::<f64>();

        let distance_from_coast = context.database_query::<f64>(&[&lon, &lat])
                                         .await?;

        usr::return_value(distance_from_coast)
      }.boxed()
    ))
  },
  FluentHandlerDefinition {
    fluent_name: "near_ports",
    dependencies: &["distance_from_ports"],
    database_query: None,
    eval_fn: EvalFn::specify(Box::new(
      |fluents, _| async move {
        let distance_from_ports = fluents.get(0)?.value::<f64>();
        usr::return_value(distance_from_ports <= 300.0)
      }.boxed()
    ))
  },
  FluentHandlerDefinition {
    fluent_name: "distance_from_ports",
    dependencies: &["lon", "lat"],
    database_query: Some(indoc! {r#"
      -- requires two input values: [lon, lat]
      select ST_Distance(
        ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857),
        ST_Transform(geom, 3857)
      ) as distance
      from ports.ports_of_brittany
      order by distance
      limit 1
    "#}),
    eval_fn: EvalFn::specify(Box::new(
      |fluents, context| async move {
        let lon = fluents.get(0)?.value::<f64>();
        let lat = fluents.get(1)?.value::<f64>();

        let distance_from_ports = context.database_query::<f64>(&[&lon, &lat])
                                         .await?;

        usr::return_value(distance_from_ports)
      }.boxed()
    ))
  },
  FluentHandlerDefinition {
    fluent_name: "is_tug_or_pilot",
    dependencies: &["speed"],
    database_query: Some(indoc! {r#"
      -- requires one input value: [key]
      select shiptype as ship_type
      from ais_data.static_ships
      where sourcemmsi = $1
      limit 1
    "#}),
    eval_fn: EvalFn::specify(Box::new(
      |fluents, context| async move {
        info!("looking for ship type");
        let key = fluents.get(0)?.keys().get(0)?.to_owned() as i32;
        info!("found key '{}'", key);
        let ship_type = context.database_query::<i32>(&[&key]).await?;
        info!("found ship type '{}' for key '{}'", ship_type, key);

        // Vessel Type Codes:
        // - 30: Fishing
        // - 31: Tug
        // - 32: Tug
        // - 35: Military
        // - 50: Pilot
        // - 51: SAR Vessel
        // - 52: Tug
        // - 53: Port Tender
        // - 55: Law Enforcement
        let type_codes = vec![31, 32, 50, 52];

        usr::return_value(type_codes.contains(&ship_type))
      }.boxed()
    ))
  }
]
