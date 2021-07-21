use std::net::IpAddr;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Run the webalert daemon
    #[structopt(alias = "s")]
    Server(ServerOpts),
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
        default_value = "postgresql://webalert:webalert@localhost/webalert_development"
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
