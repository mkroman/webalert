use crate::cli;
use crate::database::DbPool;

use tonic::transport::Server;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{debug, instrument};

pub mod v1;

#[instrument(skip(opts, _db_pool), fields(host = %opts.grpc_host))]
pub async fn start_grpc_server(opts: &cli::ServerOpts, _db_pool: DbPool) {
    debug!("Starting GRPC server");

    let layer = tower::ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_grpc().make_span_with(DefaultMakeSpan::new().include_headers(true)),
        )
        .into_inner();

    Server::builder()
        .layer(layer)
        .add_service(v1::create_runners_service())
        .serve(opts.grpc_host)
        .await
        .unwrap();
}
