[
  // (
  //   "location",
  //   vec!["lon", "lat"],
  //   false, // requires database client
  //   EvalFn::specify(Box::new(
  //     |fluents| {
  //       let (lon, lat) = match fluents.as_slice() {
  //         [Fluent::FloatPt(lon), Fluent::FloatPt(lat)] => (lon, lat),
  //         _ => panic!(),
  //       };
  //       Box::new((lon.value().to_owned(), lat.value().to_owned()))
  //     }
  //   ))
  // ),
  FluentHandlerDefinition {
    name: "high_speed",
    dependencies: &["speed"],
    database_query: None,
    eval_fn: EvalFn::specify(Box::new(
      |fluents, _| async move {
        // unwrap here is safe:
        // we are guaranteed to have what we put in the dependencies list
        let speed = fluents.get(0).unwrap();

        // optional check - fluents arrive in order given by dependency list
        if speed.name() != "speed" || !matches!(speed, Fluent::FloatPt(_)) {
          panic!();
        }

        Box::new(speed.value::<f64>() >= 5.0) as Box<dyn ValueType>
      }.boxed()
    ))
  },
  FluentHandlerDefinition {
    name: "moving",
    dependencies: &["speed"],
    database_query: None,
    eval_fn: EvalFn::specify(Box::new(
      |fluents, _| async move {
        // unwrap here is safe:
        // we are guaranteed to have what we put in the dependencies list
        let speed = fluents.get(0).unwrap();

        // optional check - fluents arrive in order given by dependency list
        if speed.name() != "speed" || !matches!(speed, Fluent::FloatPt(_)) {
          panic!();
        }

        Box::new(speed.value::<f64>() >= 0.2) as Box<dyn ValueType>
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
  }
]
