#![warn(missing_docs)]

//! A scalable webalert runner that performs actions through a WebDriver.

use std::env;

use color_eyre::{eyre::WrapErr, Report};
use structopt::StructOpt;
use tracing::{debug, trace};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

mod cli;
mod error;
mod grpc;
mod runner;

use error::{Error, Kind};

async fn async_main() -> Result<(), Report> {
    let opts = cli::Opts::from_args();

    trace!(
        %opts.grpc_url,
        %opts.webdriver_url,
        "Application started");

    debug!("Starting runner");

    let mut runner = runner::Runner::new(opts.grpc_url, opts.grpc_token, opts.webdriver_url)?;

    runner.announce().await;

    Ok(())
}

fn main() -> Result<(), Report> {
    // Override RUST_LOG with a default setting if it's not set by the user
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "webalert_runner=trace");
    }

    color_eyre::install()?;

    let fmt = tracing_subscriber::fmt::layer();
    let filter = EnvFilter::from_default_env();
    let collector = tracing_subscriber::Registry::default()
        .with(ErrorLayer::default())
        .with(filter)
        .with(fmt);

    tracing::subscriber::set_global_default(collector)
        .with_context(|| "setting global collector")?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_main())
}
