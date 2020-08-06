pub mod cli;
pub mod database;
pub mod http;

pub mod migrations {
    include!(concat!(env!("OUT_DIR"), "/migrations.rs"));
}

#[macro_use]
pub mod migration;
