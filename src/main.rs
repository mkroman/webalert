use std::time::Duration;

mod cli;

use log::debug;
use selenium_rs::webdriver::{Browser, WebDriver};
use structopt::StructOpt;

fn main() {
    env_logger::init();

    let opts = cli::Opts::from_args();

    debug!("Opts: {:?}", opts);

    for _ in 0..opts.num_webdrivers {
        let t = std::thread::spawn(|| {
            let mut driver = WebDriver::new(Browser::Chrome);
            driver.start_session().unwrap();

            debug!("Navigating to rust website");
            driver.navigate("https://www.rust-lang.org").unwrap();

            debug!("title: {:?}", driver.get_title());

            assert_eq!(
                driver.get_current_url().unwrap(),
                String::from("https://www.rust-lang.org/")
            );
        });
    }

    std::thread::sleep(Duration::from_secs(30));

    println!("Hello, world!");
}
