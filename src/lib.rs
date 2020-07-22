pub mod cli;
pub mod database;
pub mod http;

pub mod migrations {
    include!(concat!(env!("OUT_DIR"), "/migrations.rs"));
}

pub mod migration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_migrations() {
        let (up, down) = migrations::init();
        assert!(!up.is_empty());
        assert!(!down.is_empty());
    }
}
