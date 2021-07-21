use crate::cli;

use sqlx::PgPool;
use tracing::{debug, error, instrument};
use warp::Filter;

mod api_v1;

#[instrument(skip(opts, db_pool), fields(host = %opts.http_host))]
pub async fn start_http_server(opts: &cli::ServerOpts, db_pool: PgPool) {
    let addr = opts.http_host;

    debug!("Starting HTTP server");

    // GET /api/…
    let api = warp::path("api");

    // GET /api/v1
    let api_v1 = api.and(warp::path("v1"));

    // GET /api/v1/_internal/…
    let internal = api_v1.and(warp::path("_internal"));

    // GET /api/v1/_internal/runners
    let runners = internal.and(api_v1::runners(db_pool.clone()));

    // GET /api/v1/{alerts,}
    let alerts = api_v1.and(api_v1::alerts(db_pool.clone()));

    let routes = warp::any()
        .and(alerts)
        .or(runners)
        .recover(api_v1::handle_rejection);

    warp::serve(routes).run(addr).await;

    error!("server quit unexpectedly");
}
