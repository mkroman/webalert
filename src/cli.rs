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
    Up(MigrateUpOpts),
    /// Performs a rollback to the specified version
    Down(MigrateDownOpts),
}

#[derive(StructOpt, Debug)]
pub enum DbSubCommand {
    /// Perform migrations on the database
    Migrate(MigrateCommand),
}

/// Migration CLI options in the down direction where `version` is not an optional argument
#[derive(StructOpt, Debug)]
pub struct MigrateDownOpts {
    /// The version to migrate to
    pub version: String,
}

/// Migration CLI options in the up direction where `version` is either a String or None - if it is
/// None, the migration will continue until the latest available option
#[derive(StructOpt, Debug)]
pub struct MigrateUpOpts {
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
        env = "POSTGRES_URL",
        value_name = "HOSTNAME",
        default_value = "postgresql://webalert@localhost/webalert_development"
    )]
    pub postgres_url: String,
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
