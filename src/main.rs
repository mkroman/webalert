use std::time::Duration;

mod cli;
mod database;

use log::{debug, error};
use selenium_rs::webdriver::{Browser, WebDriver};
use structopt::StructOpt;
use tokio::runtime::Runtime;

/// Creates a new web driver with a started session
fn _create_driver() -> Result<WebDriver, Box<dyn std::error::Error>> {
    let mut driver = WebDriver::new(Browser::Chrome);

    driver.start_session()?;

    Ok(driver)
}

async fn async_main(opts: cli::ServerOpts) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the PostgreSQL database
    let conf = database::postgres_config_from_server_opts(opts)?;
    let conn = database::connect(conf).await?;

    // Create migration table if it doesn't exist
    database::init(&conn).await?;

    Ok(())
}

fn main() {
    env_logger::init();

    // Set up the async runtime
    let mut rt = Runtime::new().unwrap();
    // Parse the command-line arguments
    let opts = cli::Opts::from_args();

    match opts {
        cli::Opts::Server(opts) => {
            if let Err(err) = rt.block_on(async_main(opts)) {
                error!("runtime error: {}", err);
            }
        }
        _ => {}
    }
}
