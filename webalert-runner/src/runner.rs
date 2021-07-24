//! Asynchronous runner that talks to a server

use std::sync::Arc;
use std::time::Duration;

use http::HeaderValue;
use tonic::transport::Channel;
use url::Url;

use crate::grpc::{runner::AnnounceRequest, AuthService, RunnerClient};
use crate::{Error, Kind};

pub struct Runner {
    /// The host and port of the webalert gRPC server
    grpc_url: String,
    /// The authorization token to access the gRPC server
    grpc_token: String,
    /// The url to the webdriver we're operating
    webdriver_url: Url,
    // The inner gRPC client
    client: RunnerClient<AuthService<Channel>>,
}

impl Runner {
    /// Creates a new runner with the given `grpc_url`, `grpc_token` and `webdriver_url`.
    pub fn new(grpc_url: String, grpc_token: String, webdriver_url: Url) -> Result<Runner, Error> {
        let channel = Channel::from_shared(grpc_url.clone())
            .map_err(|_| Error::from(Kind::InvalidRpcUrl))?
            .connect_timeout(Duration::from_secs(30))
            .connect_lazy()?;
        let token = HeaderValue::from_str(&format!("Bearer {}", grpc_token)).unwrap();
        let client = RunnerClient::new(AuthService::new(channel, Arc::new(token)));

        Ok(Runner {
            grpc_url,
            grpc_token,
            webdriver_url,
            client,
        })
    }

    /// Announces to the gRPC server that this runner is alive and running.
    ///
    /// See also [`RunnerClient::announce`]
    pub async fn announce(&mut self) {
        use std::env::consts;

        self.client
            .announce(AnnounceRequest {
                os: consts::OS.to_string(),
                arch: consts::ARCH.to_string(),
                hostname: hostname::get()
                    .expect("could not get system hostname")
                    .into_string()
                    .unwrap(),
            })
            .await;
    }
}
