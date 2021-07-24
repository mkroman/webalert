//! gRPC client and authentication types

pub mod auth;
pub mod runner {
    tonic::include_proto!("webalert.runner.v1");
}

pub use auth::AuthService;
pub use runner::runner_client::RunnerClient;
