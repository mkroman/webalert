use std::ffi::OsStr;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

use caps::CapSet;
use thirtyfour::prelude::DesiredCapabilities;
use thirtyfour::WebDriver;
use tracing::debug;

use crate::{Error, Kind};

/// Spawns a ChromeDriver process
#[derive(Debug)]
pub struct ChromeDriver {
    child: Child,
    port: u16,
}

impl ChromeDriver {
    /// Opens a new ChromeDriver by executing `program` and setting the given `port`.
    pub fn new<S: AsRef<OsStr>>(program: S, port: u16) -> Result<ChromeDriver, Error> {
        debug!("Starting new chromedriver process");

        let mut cmd = Command::new(program);
        let cmd = unsafe {
            cmd.pre_exec(|| {
                // Drop process capabilities
                debug!("Clearing effective process capabilities");

                caps::clear(None, CapSet::Effective)
                    .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

                Ok(())
            })
        };

        let cmd = cmd.arg(format!("--port={}", port));

        debug!(?cmd, "Starting background process");
        let child = cmd
            .spawn()
            .map_err(|error| Error::from(Kind::CouldNotSpawnChromeDriver(error)))?;

        Ok(ChromeDriver { child, port })
    }

    /// Opens a new WebDriver connection to the ChromeDriver.
    pub async fn webdriver(&self) -> Result<WebDriver, Error> {
        let caps = DesiredCapabilities::firefox().set_headless().unwrap();
        let driver = WebDriver::new(&format!("http://localhost:{}", self.port), &caps).await?;

        Ok(driver)
    }

    /// Returns true if the chromedriver process has exited, false otherwise.
    pub fn has_exited(&mut self) -> bool {
        if let Ok(None) = self.child.try_wait() {
            false
        } else {
            true
        }
    }

    /// Kills the chromedriver process if it's running.
    pub fn kill(&mut self) {
        self.child.kill().unwrap()
    }
}

impl Drop for ChromeDriver {
    fn drop(&mut self) {
        if self.child.try_wait().unwrap().is_none() {
            self.child.kill().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_work() {
        let res = ChromeDriver::new("chromedriver", 4444);

        assert!(res.is_ok());
    }

    #[test]
    fn it_should_fail_when_invalid_program() {
        let res = ChromeDriver::new(
            "c89bbedd94294c27c4cbf2f97e4e49dfaf62d853c7d4732533dfd35ce4f7b27b",
            4444,
        );

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn it_should_connect() {
        let cd = ChromeDriver::new("chromedriver", 4444).unwrap();
        let driver = cd.webdriver().await;

        assert!(driver.is_ok());

        driver.unwrap().quit().await.unwrap();
    }
}
