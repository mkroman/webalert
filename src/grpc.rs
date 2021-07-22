use crate::cli;
use crate::database::DbPool;

use tonic::transport::Server;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{debug, instrument};

pub mod v1;

#[instrument(skip(opts, _db_pool), fields(host = %opts.grpc_host))]
pub async fn start_grpc_server(opts: &cli::ServerOpts, _db_pool: DbPool) {
    debug!("Starting gRPC server");

    let layer = tower::ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_grpc().make_span_with(DefaultMakeSpan::new().include_headers(true)),
        )
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
