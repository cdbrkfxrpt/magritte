// Copyright 2021 Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.


#[allow(unused)]
pub fn round_f32(n: f32, d: i64) -> f32 {
  (n * (d * 10) as f32).round() / (d * 10) as f32
}

#[allow(unused)]
pub fn round_f64(n: f64, d: i64) -> f64 {
  (n * (d * 10) as f64).round() / (d * 10) as f64
}
