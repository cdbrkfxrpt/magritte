[
  // (
  //   "location",
  //   vec!["lon", "lat"],
  //   false, // requires database client
  //   EvalFn::specify(Box::new(
  //     |fluents| {
  //       let (lon, lat) = match fluents.as_slice() {
  //         [AnyFluent::FloatPt(lon), AnyFluent::FloatPt(lat)] => (lon, lat),
  //         _ => panic!(),
  //       };

  //       Box::new((lon.value().to_owned(), lat.value().to_owned()))
  //     }
  //   ))
  // ),
  (
    "highSpeed",
    vec!["speed"],
    false, // requires database client
    EvalFn::specify(Box::new(
      |fluents| {
        let speed = match fluents.as_slice() {
          [AnyFluent::FloatPt(speed)] => speed,
          _ => panic!(),
        };

        Box::new(speed.value() >= &5.0)
      }
    ))
  ),
  (
    "moving",
    vec!["speed"],
    false, // requires database client
    EvalFn::specify(Box::new(
      |fluents| {
        let speed = match fluents.as_slice() {
          [AnyFluent::FloatPt(speed)] => speed,
          _ => panic!(),
        };

        Box::new(speed.value() >= &0.2)
      }
    ))
  ),
]
