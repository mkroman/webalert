use std::fmt;
use std::io;

use tracing_error::TracedError;

/// Error wrapper that wraps an error [`Kind`] in a [`TracedError`] to preserve error spans in
/// tracing events.
#[derive(Debug, thiserror::Error)]
pub struct Error {
    source: TracedError<Kind>,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.source, fmt)
    }
}

impl<E> From<E> for Error
where
    Kind: From<E>,
{
    fn from(source: E) -> Self {
        Self {
            source: Kind::from(source).into(),
        }
    }
}

/// The possible types of errors.
#[derive(Debug, thiserror::Error)]
pub enum Kind {
    /// Occurs when the user provides an invalid gRPC URL.
    #[error("The given gRPC URL is invalid")]
    InvalidRpcUrl,
    /// Occurs when there's a general error from the tonic transport layer.
    #[error("RPC transport error")]
    RpcTransportError(#[from] tonic::transport::Error),
    /// This error occurs when the runner tries to send an announcement message but it fails for a
    /// non-specific reason.
    #[error("Could not send announce rpc message")]
    RpcAnnounceFailed,
    /// Occurs when unable to spawn a new chromedriver process
    #[error("Could not spawn a new chromedriver process")]
    CouldNotSpawnChromeDriver(#[source] io::Error),
    /// Occurs when unable to connect to the webdriver
    #[error("Could not connect to the WebDriver")]
    WebDriverConnectionFailed(#[from] thirtyfour::error::WebDriverError),
    /// Occurs when trying to spawn a chromedriver and the previous child process is still alive
    #[error("chromedriver is already running")]
    ChromeDriverAlreadyRunning,
    #[error("Could not get the hostname")]
    HostnameUnavailable,
    /// Non-specialized IO error
    #[error("IO error")]
    IoError(#[from] io::Error),
}
