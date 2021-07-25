use std::iter;
use std::task::{Context, Poll};

use crate::cli;
use crate::database::DbPool;

use http::{header, Request, Response, StatusCode};
use tonic::body::BoxBody;
use tonic::transport::{Body, Server};
use tower::{Layer, Service};
use tower_http::sensitive_headers::SetSensitiveRequestHeadersLayer;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{debug, instrument, warn};

pub mod v1;

#[derive(Debug, Clone)]
struct RequireBearerAuthorizationLayer {
    pool: DbPool,
}

impl<S> Layer<S> for RequireBearerAuthorizationLayer {
    type Service = RequireBearerAuthorization<S>;

    fn layer(&self, service: S) -> Self::Service {
        RequireBearerAuthorization { inner: service }
    }
}

#[derive(Debug, Clone)]
struct RequireBearerAuthorization<S> {
    inner: S,
}

impl<S> Service<hyper::Request<Body>> for RequireBearerAuthorization<S>
where
    S: Service<hyper::Request<Body>, Response = hyper::Response<BoxBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            // Do extra async work here...
            let response = if let Some(_auth) = req.headers().get(header::AUTHORIZATION) {
                let response = inner.call(req).await?;
                response
            } else {
                warn!("Refusing request because it doesn't have an authorization header");

                //let mut response = Response::new(tonic::body::empty_body());
                Response::builder()
                    .header(header::CONTENT_TYPE, "application/grpc")
                    .status(StatusCode::UNAUTHORIZED)
                    .body(tonic::body::empty_body())
                    .unwrap()
            };

            Ok(response)
        })
    }
}

#[instrument(skip(opts, db_pool), fields(host = %opts.grpc_host))]
pub async fn start_server(opts: &cli::ServerOpts, db_pool: DbPool) {
    debug!("Starting gRPC server");

    // Build the bearer token authorization layer
    let auth_layer = RequireBearerAuthorizationLayer {
        pool: db_pool.clone(),
    };

    let layer = tower::ServiceBuilder::new()
        // Mark the `Authorization` request header as sensitive so it doesn't show in logs
        .layer(SetSensitiveRequestHeadersLayer::new(iter::once(
            header::AUTHORIZATION,
        )))
        // High level logging of requests and responses
        .layer(
            TraceLayer::new_for_grpc().make_span_with(DefaultMakeSpan::new().include_headers(true)),
        )
        .layer(auth_layer)
        .into_inner();

    // Build the reflection service
    let mut ref_svc = tonic_reflection::server::Builder::configure();

    for fds in v1::file_descriptor_sets() {
        ref_svc = ref_svc.register_encoded_file_descriptor_set(fds);
    }

    Server::builder()
        .layer(layer)
        .add_service(ref_svc.build().unwrap())
        .add_service(v1::create_runners_service())
        .serve(opts.grpc_host)
        .await
        .unwrap();
}
