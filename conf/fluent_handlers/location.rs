(
  &["lon", "lat"],
  |values: Vec<AnyFluent>| {
    let AnyFluent::FloatPt(lon) = &values[0] else { panic!() };
    let AnyFluent::FloatPt(lat) = &values[1] else { panic!() };

    (lon.value().to_owned(), lat.value().to_owned())
  }
)
