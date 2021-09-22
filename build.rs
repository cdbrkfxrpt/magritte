// Copyright 2021 Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

pub fn main() {
  // we'd like to see our protocol buffer syntax errors, so this panics and
  // dumps the original error message if we made one.
  if let Err(err) = tonic_build::compile_protos("proto/dylonet.proto") {
    panic!("FATAL | {}", err);
  }
}
