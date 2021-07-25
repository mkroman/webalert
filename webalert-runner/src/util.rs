pub mod system {
    use std::env::consts;

    use crate::error::{Error, Kind};

    /// Returns a string describing the current system operating system.
    pub fn get_os() -> String {
        consts::OS.to_string()
    }

    /// Returns a string describing the current system architecture.
    pub fn get_arch() -> String {
        consts::ARCH.to_string()
    }

    /// Returns the systems current hostname.
    ///
    /// # Errors
    ///
    /// Returns an error with kind [`Kind::HostnameUnavailable`] if the hostname is inaccessible.
    pub fn get_hostname() -> Result<String, Error> {
        match hostname::get() {
            Ok(hostname) => Ok(hostname.as_os_str().to_string_lossy().into_owned()),
            Err(_) => Err(Error::from(Kind::HostnameUnavailable)),
        }
    }
}
