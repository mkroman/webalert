use log::{debug, error};
use warp::Filter;

pub async fn create_http_server() {
    let hello =
        warp::get().and(warp::path!("hello" / String).map(|name| format!("Hello, {}!", name)));
    let not_found = warp::any().map(|| "Hello, world!");

    debug!("Starting HTTP server");

    let routes = warp::any().and(hello.or(not_found));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    error!("server quit unexpectedly");
}
