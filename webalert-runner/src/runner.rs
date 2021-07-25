//! Asynchronous runner that talks to a server

use std::io;
use std::sync::Arc;
use std::time::Duration;

use caps::CapSet;
use http::HeaderValue;
use tokio::{
    process::{Child, Command},
    time,
};
use tonic::{transport::Channel, Code, Status};
use tracing::{debug, error, instrument};

use crate::grpc::{runner::AnnounceRequest, AuthService, RunnerClient};
use crate::util::system;
use crate::{Error, Kind};

/// Asynchronous client that communicates with a gRPC server and receives tasks to run in a
/// webdriver.
pub struct Runner {
    /// The host and port of the webalert gRPC server
    grpc_url: String,
    /// The authorization token to access the gRPC server
    grpc_token: String,
    /// A handle to the webdriver child process once launched
    webdriver_child: Option<Child>,
    // The inner gRPC client
    client: RunnerClient<AuthService<Channel>>,
}

/// Spawns a new chromedriver process and returns the child handle.
///
/// # Errors
///
/// Returns an error with type [`Kind::CouldNotSpawnChromeDriver`] when the process fails to
/// execute.
#[instrument]
fn spawn_chromedriver_proc() -> Result<Child, Error> {
    debug!("Starting new chromedriver process");

    let mut cmd = Command::new("chromedriver");
    let cmd = unsafe {
        cmd.pre_exec(|| {
            // Drop process capabilities
            caps::clear(None, CapSet::Effective)
                .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

            Ok(())
        })
    };

    let cmd = cmd.arg("--port=4444");

    debug!(?cmd, "Starting background process");
    let child = cmd
        .spawn()
        .map_err(|error| Error::from(Kind::CouldNotSpawnChromeDriver(error)))?;

    Ok(child)
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
            webdriver_child: None,
            client,
        })
    }

    /// Spawns a new `chromedriver` process in the background.
    pub fn spawn_chromedriver(&mut self) -> Result<(), Error> {
        if let Some(ref mut child) = self.webdriver_child {
            // If we can't get an exit status from `try_wait()` it means the process hasn't exited
            if let Ok(None) = child.try_wait() {
                return Err(Error::from(Kind::ChromeDriverAlreadyRunning));
            }
        }

        let child = spawn_chromedriver_proc()?;
        self.webdriver_child = Some(child);

        Ok(())
    }

    /// Continually polls the server for new tasks.
    #[instrument(skip(self))]
    pub async fn poll(&mut self) -> Result<(), Error> {
        loop {
            let result = self.client.poll(()).await;

            match result {
                Ok(response) => {
                    debug!(?response, "Received poll response");
                }
                Err(error) => {
                    if error.code() != Code::NotFound {
                        error!(?error, "Poll failed");

                        break;
                    }
                }
            }

            time::sleep(Duration::from_secs(1)).await;
        }

        if let Some(ref mut child) = self.webdriver_child {
            debug!(child.pid = ?child.id(), "Killing webdriver process");

            child.kill().await?;
        }

        Ok(())
    }

    /// Stops the chromedriver process.
    pub async fn stop(&mut self) -> Result<(), Error> {
        if let Some(ref mut child) = self.webdriver_child {
            child.kill().await?;
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
mod tests {
    use super::*;

    #[tokio::test]
    async fn limit_to_one_chromedriver_process() -> Result<(), Box<dyn std::error::Error>> {
        let mut runner = Runner::new("http://localhost:3030".to_string(), "test".to_string())?;

        assert!(runner.spawn_chromedriver().is_ok());
        assert!(runner.spawn_chromedriver().is_err());

        // Gracefully exit to avoid dangling process
        runner.stop().await?;

        Ok(())
    }
}
