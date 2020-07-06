use std::net::SocketAddr;

use crate::cli;

use log::{debug, error};
use warp::{http::header::HeaderValue, http::Response, Filter, Reply};

mod api_v1;

fn not_found() -> impl Reply {
    Response::builder()
        .status(404)
        .body("The requested resource was not found")
}

pub async fn start_http_server(opts: &cli::ServerOpts) {
    debug!("Starting HTTP server");

    let addr: SocketAddr = (opts.host, opts.port).into();
    debug!("Binding to {}", addr);

    // GET /api/…
    let api = warp::path("api");

    // GET /api/v1/…
    let api_v1 = api.and(warp::path("v1").and(warp::header::value("x-token")).map(
        |value: HeaderValue| {
            println!("Received token: {:?}", value);

            "henlo"
        },
    ));

    // 404 error handler
    let not_found_error = warp::any().map(not_found);

    let routes = warp::any().and(api_v1.or(not_found_error));

    warp::serve(routes).run(addr).await;

    error!("server quit unexpectedly");
}
