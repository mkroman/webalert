use webalert::http;
use webalert::{cli, database};

use log::{debug, error};
use structopt::StructOpt;
use tokio::runtime::Runtime;

async fn async_main(opts: cli::Opts) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the PostgreSQL database
    let pool = database::init(&opts).await?;

    match &opts.command {
        cli::Command::Server(ref server_opts) => {
            debug!("Starting server");

            let http_server = http::start_http_server(server_opts, pool);

            tokio::join!(http_server);
        }
    }

    Ok(())
}

fn main() {
    env_logger::init();

    // Set up the async runtime
    let rt = Runtime::new().expect("unable to create runtime");
    // Parse the command-line arguments
    let opts = cli::Opts::from_args();

    match &opts.command {
        cli::Command::Server(_) => {
            if let Err(err) = rt.block_on(async_main(opts)) {
                error!("runtime error: {}", err);
            }
        }
    }
}
