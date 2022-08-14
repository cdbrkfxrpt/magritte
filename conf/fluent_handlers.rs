[
  (
    "location",
    vec!["lon", "lat"],
    |values: Vec<AnyFluent>| {
      let (lon, lat) = match values.as_slice() {
        [AnyFluent::FloatPt(lon), AnyFluent::FloatPt(lat)] => (lon, lat),
        _ => panic!(),
      };

      (lon.value().to_owned(), lat.value().to_owned())
    }
  ),
  (
    "highSpeed",
    vec!["speed"],
    |values: Vec<AnyFluent>| {
      let speed = match values.as_slice() {
        [AnyFluent::FloatPt(speed)] => speed,
        _ => panic!(),
      };

      speed.value() >= &5.0
    }
  )
]
