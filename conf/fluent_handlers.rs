[
  FluentHandlerDefinition {
    name: "high_speed_near_coast",
    dependencies: &["high_speed", "near_coast"],
    database_query: None,
    eval_fn: EvalFn::specify(Box::new(
      |fluents, _| async move {
        // unwrap here is safe:
        // we are guaranteed to have what we put in the dependencies list
        let high_speed = fluents.get(0).unwrap().value::<bool>();
        let near_coast = fluents.get(1).unwrap().value::<bool>();

        Box::new(high_speed && near_coast) as Box<dyn ValueType>
      }.boxed()
    ))
  },
  FluentHandlerDefinition {
    name: "high_speed",
    dependencies: &["speed"],
    database_query: None,
    eval_fn: EvalFn::specify(Box::new(
      |fluents, _| async move {
        // unwrap here is safe:
        // we are guaranteed to have what we put in the dependencies list
        let speed_fluent = fluents.get(0).unwrap();
        // optional check - fluents arrive in order given by dependency list
        if speed_fluent.name() != "speed"
           || !matches!(speed_fluent, Fluent::FloatPt(_)) {
          panic!();
        }

        let speed = speed_fluent.value::<f64>();
        Box::new(speed >= 5.0) as Box<dyn ValueType>
      }.boxed()
    ))
  },
  FluentHandlerDefinition {
    name: "near_coast",
    dependencies: &["distance_from_coast"],
    database_query: None,
    eval_fn: EvalFn::specify(Box::new(
      |fluents, _| async move {
        // unwrap here is safe:
        // we are guaranteed to have what we put in the dependencies list
        let distance_from_coast = fluents.get(0).unwrap().value::<f64>();

        Box::new(distance_from_coast < 300.0) as Box<dyn ValueType>
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
        // unwrap here is safe:
        // we are guaranteed to have what we put in the dependencies list
        let lon = fluents.get(0).unwrap().value::<f64>();
        let lat = fluents.get(1).unwrap().value::<f64>();

        let distance_from_coast = context.database_query::<f64>(&[&lon, &lat])
                                         .await
                                         .unwrap();
        Box::new(distance_from_coast) as Box<dyn ValueType>
      }.boxed()
    ))
  },
]
