// Copyright 2021 Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use magritte::{DataPoint, State, StateResponse};

use eyre::Result;
use tracing::info;


#[derive(Default, Debug)]
pub struct Dylonet {
  state:    State,
  message:  String,
  accessed: u64,
  // netindex: ?
}

impl Dylonet {
  pub fn new() -> Self {
    info!("Dylonet spawning");
    Self { state:    State::Ready,
           message:  String::new(),
           accessed: 0u64, }
  }

  pub fn receive(&mut self, datapoint: DataPoint) -> Result<()> {
    info!("received datapoint: {:?}", datapoint);
    self.message = format!("received datapoint | id: {}", datapoint.id);
    Ok(())
  }

  pub fn as_state_response(&mut self) -> StateResponse {
    self.accessed += 1;

    info!("responding: {:?}", self);
    StateResponse { state:    self.state as i32,
                    msg:      self.message.clone(),
                    accessed: self.accessed, }
  }
}
