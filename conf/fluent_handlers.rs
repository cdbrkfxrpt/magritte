[
  (
    "location",
    &["lon", "lat"],
    Fn!(
      |values: Vec<AnyFluent>| {
        let (lon, lat) = match values.as_slice() {
          [AnyFluent::FloatPt(lon), AnyFluent::FloatPt(lat)] => (lon, lat),
          _ => panic!(),
        };

        (lon.value().to_owned(), lat.value().to_owned())
    }
    )
  )
]
