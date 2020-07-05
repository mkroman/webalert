pub mod cli;
pub mod database;
pub mod http_server;

pub mod migrations {
    include!(concat!(env!("OUT_DIR"), "/migrations.rs"));
}

pub mod migration;
