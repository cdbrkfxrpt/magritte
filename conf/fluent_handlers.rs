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
      }.boxed())
    )
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
      }.boxed())
    )
  },
]
