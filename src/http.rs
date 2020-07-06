use std::net::SocketAddr;
use std::sync::Arc;

use crate::cli;

use log::{debug, error};
use tokio_postgres::Client;
use warp::Filter;

mod api_v1;

pub async fn start_http_server(opts: &cli::ServerOpts, db: Client) {
    debug!("Starting HTTP server");

    let addr: SocketAddr = (opts.host, opts.port).into();
    debug!("Binding to {}", addr);

    // GET /api/…
    let api = warp::path("api");

    // GET /api/v1
    let api_v1 = api.and(warp::path("v1"));

    // GET /api/v1/{alerts,}
    let alerts = api_v1.and(api_v1::alerts(Arc::new(db)));

    let routes = warp::any().and(alerts).recover(api_v1::handle_rejection);

    warp::serve(routes).run(addr).await;

    error!("server quit unexpectedly");
}
