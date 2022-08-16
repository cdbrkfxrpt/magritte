[
  (
    "location",
    vec!["lon", "lat"],
    EvalFn::specify(Box::new(
      |fluents: Vec<AnyFluent>| {
        let (lon, lat) = match fluents.as_slice() {
          [AnyFluent::FloatPt(lon), AnyFluent::FloatPt(lat)] => (lon, lat),
          _ => panic!(),
        };

        Box::new((lon.value().to_owned(), lat.value().to_owned()))
      }
    ))
  ),
  (
    "moving",
    vec!["speed"],
    EvalFn::specify(Box::new(
      |fluents: Vec<AnyFluent>| {
        let speed = match fluents.as_slice() {
          [AnyFluent::FloatPt(speed)] => speed,
          _ => panic!(),
        };

        Box::new(speed.value() >= &0.2)
      }
    ))
  ),
  (
    "highSpeed",
    vec!["speed"],
    EvalFn::specify(Box::new(
      |fluents: Vec<AnyFluent>| {
        let speed = match fluents.as_slice() {
          [AnyFluent::FloatPt(speed)] => speed,
          _ => panic!(),
        };

        Box::new(speed.value() >= &5.0)
      }
    ))
  ),
]
