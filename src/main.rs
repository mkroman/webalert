use std::time::Duration;

mod cli;
mod database;

use log::debug;
use selenium_rs::webdriver::{Browser, WebDriver};
use structopt::StructOpt;
use tokio::runtime::Runtime;

/// Creates a new web driver with a started session
fn create_driver() -> Result<WebDriver, Box<dyn std::error::Error>> {
    let mut driver = WebDriver::new(Browser::Chrome);

    driver.start_session()?;

    Ok(driver)
}

async fn async_main(opts: cli::Opts) -> Result<(), Box<dyn std::error::Error>> {
    let conf = database::postgres_config_from_opts(opts)?;
    let conn = database::connect(conf).await?;

    Ok(())
}

fn main() {
    env_logger::init();

    // Set up the async runtime
    let mut rt = Runtime::new().unwrap();

    let opts = cli::Opts::from_args();

    debug!("Opts: {:?}", opts);

    /*
    for _ in 0..opts.num_webdrivers {
        std::thread::spawn(|| {
            let mut driver = create_driver().expect("could not create driver");

            debug!("Navigating to rust website");
            driver.navigate("https://www.rust-lang.org").unwrap();

            debug!("title: {:?}", driver.get_title());

            assert_eq!(
                driver.get_current_url().unwrap(),
                String::from("https://www.rust-lang.org/")
            );
        });
    }*/

    rt.block_on(async_main(opts)).unwrap();
}
