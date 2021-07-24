use std::fmt;

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
}
