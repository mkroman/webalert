//! Asynchronous runner that talks to a server

use std::sync::Arc;
use std::time::Duration;

use http::HeaderValue;
use tonic::{transport::Channel, Status};
use tracing::{debug, error, instrument};

use crate::grpc::{runner::AnnounceRequest, AuthService, RunnerClient};
use crate::util::system;
use crate::webdriver::ChromeDriver;
use crate::{Error, Kind};

/// Asynchronous client that communicates with a gRPC server and receives tasks to run in a
/// webdriver.
pub struct Runner {
    /// The host and port of the webalert gRPC server
    grpc_url: String,
    /// The authorization token to access the gRPC server
    grpc_token: String,
    /// A handle to the webdriver child process once launched
    chromedriver: Option<ChromeDriver>,
    // The inner gRPC client
    client: RunnerClient<AuthService<Channel>>,
}

impl Runner {
    /// Creates a new runner with the given `grpc_url`, `grpc_token` and `webdriver_url`.
    #[instrument(skip(grpc_token))]
    pub fn new(grpc_url: String, grpc_token: String) -> Result<Runner, Error> {
        let channel = Channel::from_shared(grpc_url.clone())
            .map_err(|_| Error::from(Kind::InvalidRpcUrl))?
            .connect_timeout(Duration::from_secs(30))
            .connect_lazy()?;
        let token = HeaderValue::from_str(&format!("Bearer {}", grpc_token)).unwrap();
        let client = RunnerClient::new(AuthService::new(channel, Arc::new(token)));

        Ok(Runner {
            grpc_url,
            grpc_token,
            chromedriver: None,
            client,
        })
    }

    /// Spawns a new `chromedriver` process in the background.
    pub fn spawn_chromedriver(&mut self) -> Result<(), Error> {
        if let Some(ref mut chromedriver) = self.chromedriver {
            // If we can't get an exit status from `try_wait()` it means the process hasn't exited
            if !chromedriver.has_exited() {
                return Err(Error::from(Kind::ChromeDriverAlreadyRunning));
            }
        }

        let chromedriver = ChromeDriver::new("chromedriver", 4444)?;
        self.chromedriver = Some(chromedriver);

        Ok(())
    }

    /// Continually polls the server for new tasks.
    #[instrument(skip(self))]
    pub async fn poll(&mut self) -> Result<(), Error> {
        let mut stream = self.client.poll(()).await.unwrap().into_inner();

        loop {
            let msg = stream.message().await;

            match msg {
                Ok(Some(msg)) => {
                    println!("{:?}", msg);
                }
                Ok(None) => unreachable!(),
                Err(err) => {
                    error!(?err);

                    break;
                }
            }
        }

        debug!("Polling stream ended");

        if let Some(ref mut chromedriver) = self.chromedriver {
            if !chromedriver.has_exited() {
                debug!(?chromedriver, "Killing webdriver process");

                chromedriver.kill();
            }
        }

        Ok(())
    }

    /// Stops the chromedriver process.
    pub async fn stop(&mut self) -> Result<(), Error> {
        if let Some(ref mut chromedriver) = self.chromedriver {
            chromedriver.kill();
        }

        Ok(())
    }

    /// Announces to the gRPC server that this runner is alive and running.
    ///
    /// See also [`RunnerClient::announce`]
    #[instrument(skip(self))]
    pub async fn announce(&mut self) -> Result<(), Status> {
        let hostname =
            system::get_hostname().map_err(|error| Status::internal(error.to_string()))?;

        self.client
            .announce(AnnounceRequest {
                os: system::get_os(),
                arch: system::get_arch(),
                hostname,
            })
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
