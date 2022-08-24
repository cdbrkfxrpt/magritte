[
  // FluentHandlerDefinition {
  //   fluent_name: "high_speed_near_coast",
  //   dependencies: &["high_speed", "near_coast"],
  //   key_dependency: KeyDependency::Concurrent,
  //   database_query: None,
  //   eval_fn: EvalFn::specify(Box::new(
  //     |fluents, _| async move {
  //       let high_speed = fluents.get(0)?.value::<bool>();
  //       let near_coast = fluents.get(1)?.value::<bool>();

  //       usr::return_value(high_speed && near_coast)
  //     }.boxed()
  //   )),
  //   prune_after: 3600,
  // },
  // FluentHandlerDefinition {
  //   fluent_name: "high_speed",
  //   dependencies: &["speed"],
  //   key_dependency: KeyDependency::Concurrent,
  //   database_query: None,
  //   eval_fn: EvalFn::specify(Box::new(
  //     |fluents, _| async move {
  //       let speed_fluent = fluents.get(0)?;

  //       // optional check - fluents arrive in order given by dependency list
  //       if speed_fluent.name() != "speed"
  //          || !matches!(speed_fluent, Fluent::FloatPt(_)) {
  //         panic!();
  //       }

  //       let speed = speed_fluent.value::<f64>();
  //       usr::return_value(speed > 5.0)
  //     }.boxed()
  //   )),
  //   prune_after: 3600,
  // },
  // FluentHandlerDefinition {
  //   fluent_name: "near_coast",
  //   dependencies: &["distance_from_coast"],
  //   key_dependency: KeyDependency::Concurrent,
  //   database_query: None,
  //   eval_fn: EvalFn::specify(Box::new(
  //     |fluents, _| async move {
  //       let distance_from_coast = fluents.get(0)?.value::<f64>();
  //       usr::return_value(distance_from_coast <= 300.0)
  //     }.boxed()
  //   )),
  //   prune_after: 3600
  // },
  // FluentHandlerDefinition {
  //   fluent_name: "distance_from_coast",
  //   dependencies: &["location"],
  //   key_dependency: KeyDependency::Concurrent,
  //   database_query: Some(indoc! {r#"
  //     -- requires two input values: [lon, lat]
  //     select ST_Distance(
  //       ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857),
  //       geom
  //     ) as distance
  //     from magritte.europe_coastline
  //     limit 1
  //   "#}),
  //   eval_fn: EvalFn::specify(Box::new(
  //     |fluents, context| async move {
  //       let (lon, lat) = fluents.get(0)?.value::<(f64, f64)>();

  //       let distance_from_coast = context.database_query::<f64>(&[&lon, &lat])
  //                                        .await?;

  //       usr::return_value(distance_from_coast)
  //     }.boxed()
  //   )),
  //   prune_after: 3600
  // },
  // FluentHandlerDefinition {
  //   fluent_name: "rendez_vous_conditions",
  //   dependencies: &[
  //     "stopped_or_low_speed",
  //     "is_tug_or_pilot",
  //     "near_coast",
  //     "near_ports"
  //   ],
  //   key_dependency: KeyDependency::Concurrent,
  //   database_query: None,
  //   eval_fn: EvalFn::specify(Box::new(
  //     |fluents, _| async move {
  //       let stopped_or_low_speed = fluents.get(0)?.value::<bool>();
  //       let is_tug_or_pilot = fluents.get(1)?.value::<bool>();
  //       let near_coast = fluents.get(2)?.value::<bool>();
  //       let near_ports = fluents.get(3)?.value::<bool>();

  //       usr::return_value(
  //         stopped_or_low_speed
  //         && !is_tug_or_pilot
  //         && !near_coast
  //         && !near_ports
  //       )
  //     }.boxed()
  //   )),
  //   prune_after: 3600
  // },
  // FluentHandlerDefinition {
  //   fluent_name: "stopped_or_low_speed",
  //   dependencies: &["speed"],
  //   key_dependency: KeyDependency::Concurrent,
  //   database_query: None,
  //   eval_fn: EvalFn::specify(Box::new(
  //     |fluents, _| async move {
  //       let speed = fluents.get(0)?.value::<f64>();
  //       usr::return_value(speed <= 5.0)
  //     }.boxed()
  //   )),
  //   prune_after: 3600
  // },
  // FluentHandlerDefinition {
  //   fluent_name: "near_ports",
  //   dependencies: &["distance_from_ports"],
  //   key_dependency: KeyDependency::Concurrent,
  //   database_query: None,
  //   eval_fn: EvalFn::specify(Box::new(
  //     |fluents, _| async move {
  //       let distance_from_ports = fluents.get(0)?.value::<f64>();
  //       usr::return_value(distance_from_ports <= 300.0)
  //     }.boxed()
  //   )),
  //   prune_after: 3600
  // },
  // FluentHandlerDefinition {
  //   fluent_name: "distance_from_ports",
  //   dependencies: &["location"],
  //   key_dependency: KeyDependency::Concurrent,
  //   database_query: Some(indoc! {r#"
  //     -- requires two input values: [lon, lat]
  //     select ST_Distance(
  //       ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857),
  //       ST_Transform(geom, 3857)
  //     ) as distance
  //     from ports.ports_of_brittany
  //     order by distance
  //     limit 1
  //   "#}),
  //   eval_fn: EvalFn::specify(Box::new(
  //     |fluents, context| async move {
  //       let (lon, lat) = fluents.get(0)?.value::<(f64, f64)>();

  //       let distance_from_ports = context.database_query::<f64>(&[&lon, &lat])
  //                                        .await?;

  //       usr::return_value(distance_from_ports)
  //     }.boxed()
  //   )),
  //   prune_after: 3600
  // },
  // FluentHandlerDefinition {
  //   fluent_name: "is_tug_or_pilot",
  //   dependencies: &["speed"],
  //   key_dependency: KeyDependency::Concurrent,
  //   database_query: Some(indoc! {r#"
  //     -- requires one input value: [key]
  //     select shiptype as ship_type
  //     from ais_data.static_ships
  //     where sourcemmsi = $1
  //     limit 1
  //   "#}),
  //   eval_fn: EvalFn::specify(Box::new(
  //     |fluents, context| async move {
  //       let key = fluents.get(0)?.keys().get(0)?.to_owned() as i32;
  //       let ship_type = context.database_query::<i32>(&[&key]).await?;

  //       // Vessel Type Codes:
  //       // - 30: Fishing
  //       // - 31: Tug
  //       // - 32: Tug
  //       // - 35: Military
  //       // - 50: Pilot
  //       // - 51: SAR Vessel
  //       // - 52: Tug
  //       // - 53: Port Tender
  //       // - 55: Law Enforcement
  //       let type_codes = vec![31, 32, 50, 52];

  //       usr::return_value(type_codes.contains(&ship_type))
  //     }.boxed()
  //   )),
  //   prune_after: 3600
  // },
  FluentHandlerDefinition {
    fluent_name: "proximity",
    dependencies: &["distance"],
    key_dependency: KeyDependency::Concurrent,
    database_query: None,
    eval_fn: EvalFn::specify(Box::new(
      |fluents, _| async move {
        let distance = fluents.get(0)?.value::<f64>();
        usr::return_value(distance <= 100.0)
      }.boxed()
    )),
    prune_after: 3600
  },
  FluentHandlerDefinition {
    fluent_name: "distance",
    dependencies: &["location"],
    key_dependency: KeyDependency::NonConcurrent,
    database_query: None,
    eval_fn: EvalFn::specify(Box::new(
      |fluents, _| async move {
        let (lon_a, lat_a) = fluents.get(0)?.value::<(f64, f64)>();
        let (lon_b, lat_b) = fluents.get(1)?.value::<(f64, f64)>();

        //
        // distance calculation taken from Rust Cookbook:
        // https://bit.ly/3AK0t24
        //
        let earth_radius_km = 6371.0_f64;

        let lat_a = lat_a.to_radians();
        let lat_b = lat_b.to_radians();

        let delta_lat = (lat_a - lat_b).to_radians();
        let delta_lon = (lon_a - lon_b).to_radians();

        let central_angle_inner =
          (delta_lat / 2.0).sin().powi(2)
          + lat_a.cos() * lat_b.cos() * (delta_lon / 2.0).sin().powi(2);
        let central_angle = 2.0 * central_angle_inner.sqrt().asin();

        let distance = earth_radius_km * central_angle;
        //
        // fin distance calculation
        //
        usr::return_value(distance)
      }.boxed()
    )),
    prune_after: 3600
  },
  FluentHandlerDefinition {
    fluent_name: "location",
    dependencies: &["lon", "lat"],
    key_dependency: KeyDependency::Concurrent,
    database_query: None,
    eval_fn: EvalFn::specify(Box::new(
      |fluents, _| async move {
        let lon = fluents.get(0)?.value::<f64>();
        let lat = fluents.get(1)?.value::<f64>();

        usr::return_value((lon, lat))
      }.boxed()
    )),
    prune_after: 3600,
  },
]
