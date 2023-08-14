#![cfg(not(tarpaulin_include))]
#![allow(dead_code, non_snake_case)]
mod astro_run;
pub mod astro_run_remote_runner;
pub mod astro_run_server;

pub use self::astro_run::*;

// Astro run server
pub use astro_run_server::{
  astro_run_service_client::AstroRunServiceClient,
  astro_run_service_server::{AstroRunService, AstroRunServiceServer},
};

// Astro run remote runner
pub use astro_run_remote_runner::{
  astro_run_remote_runner_client::AstroRunRemoteRunnerClient,
  astro_run_remote_runner_server::{AstroRunRemoteRunner, AstroRunRemoteRunnerServer},
};

pub use prost_types::Timestamp;
