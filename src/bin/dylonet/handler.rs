// Copyright 2021 Florian Eich <florian@bmc-labs.com>
//
// This work is licensed under the Apache License, Version 2.0. You should have
// received a copy of this license along with the source code. If that is not
// the case, please find one at http://www.apache.org/licenses/LICENSE-2.0.

use crate::dylonet::Dylonet;
use magritte::{dylonet_service_server::DylonetService,
               AggregateResult,
               DataPoint,
               Empty,
               StateResponse};

use eyre::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::info;


#[derive(Debug)]
pub struct Handler {
  dylonet: Arc<Mutex<Dylonet>>,
}

impl Handler {
  pub fn new() -> Self {
    info!("Handler spawning");
    Self { dylonet: Arc::new(Mutex::new(Dylonet::new())), }
  }
}

#[tonic::async_trait]
impl DylonetService for Handler {
  async fn state(&self,
                 _: Request<Empty>)
                 -> Result<Response<StateResponse>, Status> {
    let mtx = Arc::clone(&self.dylonet);
    let mut dylonet = mtx.lock().await;

    Ok(Response::new(dylonet.as_state_response()))
  }

  async fn deliver(&self,
                   request: Request<DataPoint>)
                   -> Result<Response<StateResponse>, Status> {
    let mtx = Arc::clone(&self.dylonet);
    let mut dylonet = mtx.lock().await;

    dylonet.receive(request.into_inner())
           .map_err(move |err| Status::invalid_argument(err.to_string()))?;

    Ok(Response::new(dylonet.as_state_response()))
  }

  async fn acquire(&self,
                   _request: Request<Empty>)
                   -> Result<Response<AggregateResult>, Status> {
    Ok(Response::new(AggregateResult {}))
  }
}
