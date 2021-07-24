//! Authentication for the gRPC client
//!
//! This is a service that allows the client to include an authorization token in requests.
//!
//! The Tonic client examples do the same thing by using the `with_interceptor` function, but by
//! doing this, the returned client has an inner generic type with a closure that makes it
//! difficult, if not impossible, to store it in a struct.
//!
//! This is copied from yotamofek's work:
//! <https://github.com/etcdv3/etcd-client/pull/21/files#diff-212eed958dda431dc3bf15be7424ef556201e53143bbe8b19bb80e19e22e5cd9>
//!
//! For more information, see:
//! <https://github.com/hyperium/tonic/pull/713>

use std::sync::Arc;
use std::task::{Context, Poll};

use http::{header::AUTHORIZATION, HeaderValue, Request};
use tower::Service;

/// Authentication [`Service`] that takes an [`AUTHORIZATION`] `token` and injects it into all
/// requests.
#[derive(Debug, Clone)]
pub struct AuthService<S> {
    inner: S,
    token: Arc<HeaderValue>,
}

impl<S> AuthService<S> {
    #[inline]
    pub fn new(inner: S, token: Arc<HeaderValue>) -> Self {
        Self { inner, token }
    }
}

impl<S, Body, Response> Service<Request<Body>> for AuthService<S>
where
    S: Service<Request<Body>, Response = Response>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        request
            .headers_mut()
            .insert(AUTHORIZATION, self.token.as_ref().clone());

        self.inner.call(request)
    }
}
