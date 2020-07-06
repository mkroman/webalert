use std::net::IpAddr;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Run the webalert daemon
    #[structopt(alias = "s")]
    Server(ServerOpts),

    /// Perform database operations
    #[structopt(name = "database", alias = "db")]
    DbCommand(DbSubCommand),
}

#[derive(StructOpt, Debug)]
pub enum MigrateCommand {
    /// Migrates the database to the specified version
    Up(MigrateOpts),
    /// Performs a rollback to the specified version
    Down(MigrateOpts),
}

#[derive(StructOpt, Debug)]
pub enum DbSubCommand {
    /// Perform migrations on the database
    Migrate(MigrateCommand),
}

#[derive(StructOpt, Debug)]
pub struct MigrateOpts {
    /// The version to migrate to
    pub version: Option<String>,
}

#[derive(StructOpt, Debug)]
pub struct Opts {
    #[structopt(subcommand)]
    pub command: Command,

    /// PostgreSQL host
    #[structopt(
        long,
        env = "POSTGRES_HOST",
        value_name = "HOSTNAME",
        default_value = "localhost"
    )]
    pub postgres_host: String,

    /// PostgreSQL user
    #[structopt(
        long,
        env = "POSTGRES_USER",
        value_name = "USER",
        default_value = "webalert"
    )]
    pub postgres_user: String,

    /// PostgreSQL user password
    #[structopt(long, env = "POSTGRES_PASSWORD", value_name = "PASSWORD")]
    pub postgres_password: String,

    /// PostgreSQL database name
    #[structopt(
        long,
        env = "POSTGRES_DB",
        value_name = "DATABASE",
        default_value = "webalert_development"
    )]
    pub postgres_db: String,
}

#[derive(StructOpt, Debug, Clone)]
pub struct ServerOpts {
    /// The number of webdrivers to run in parallel
    #[structopt(short = "n", default_value = "3")]
    pub num_webdrivers: u64,
    /// The local host to bind to
    #[structopt(short = "h", default_value = "::")]
    pub host: IpAddr,
    /// The local port to bind to
    #[structopt(short = "p", default_value = "3030")]
    pub port: u16,
}
