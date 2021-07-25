use std::env;

use webalert::{cli, database, grpc, http};

use structopt::StructOpt;
use tokio::runtime::Runtime;
use tracing::{debug, error};
use tracing_subscriber::EnvFilter;

async fn async_main(opts: cli::Opts) -> Result<(), Box<dyn std::error::Error>> {
    match &opts.command {
        cli::Command::Server(ref server_opts) => {
            // Connect to the PostgreSQL database
            debug!("Connecting to the database");
            let pool = database::connect(server_opts.database_url.as_str()).await?;

            debug!("Starting servers");
            let http_server = http::start_server(server_opts, pool.clone());
            let grpc_server = grpc::start_server(server_opts, pool.clone());

            tokio::join!(http_server, grpc_server);
        }
    }

    Ok(())
}

fn main() {
    // Override RUST_LOG with a default setting if it's not set by the user
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "webalert=trace,tower_http=trace");
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Set up the async runtime
    let rt = Runtime::new().expect("unable to create runtime");
    // Parse the command-line arguments
    let opts = cli::Opts::from_args();

    if let Err(err) = rt.block_on(async_main(opts)) {
        error!("runtime error: {}", err);
    }
}
