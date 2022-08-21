[
  FluentHandlerDefinition {
    name: "high_speed_near_coast",
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
    name: "high_speed",
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
    name: "near_coast",
    dependencies: &["distance_from_coast"],
    database_query: None,
    eval_fn: EvalFn::specify(Box::new(
      |fluents, _| async move {
        let distance_from_coast = fluents.get(0)?.value::<f64>();
        usr::return_value(distance_from_coast < 300.0)
      }.boxed()
    ))
  },
  FluentHandlerDefinition {
    name: "distance_from_coast",
    dependencies: &["lon", "lat"],
    database_query: Some(indoc! {r#"
      -- requires two input values: [lon, lat]
      select ST_Distance(
        ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857),
        geom
      ) as distance
      from
        magritte.europe_coastline
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
]
