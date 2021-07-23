use std::net::SocketAddr;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Command {
    /// Run the webalert daemon
    #[structopt(alias = "s")]
    Server(ServerOpts),

    /// Start a new runner
    #[structopt(alias = "r")]
    Runner(RunnerOpts),
}

#[derive(StructOpt, Debug)]
pub struct Opts {
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt, Debug, Clone)]
pub struct ServerOpts {
    /// The local host to bind to
    #[structopt(
        long = "http-server-host",
        env = "WEBALERT_HTTP_HOST",
        default_value = "[::]:3030"
    )]
    pub http_host: SocketAddr,

    /// GRPC host to bind to
    #[structopt(
        long = "grpc-server-host",
        env = "WEBALERT_GRPC_HOST",
        default_value = "[::]:3031"
    )]
    pub grpc_host: SocketAddr,

    /// PostgreSQL host
    #[structopt(
        long,
        env = "DATABASE_URL",
        default_value = "postgresql://webalert@localhost/webalert_development"
    )]
    pub postgres_url: String,
}

#[derive(StructOpt, Debug, Clone)]
pub struct RunnerOpts {
    /// GRPC host to connect to
    #[structopt(
        long = "grpc-server-host",
        env = "WEBALERT_GRPC_HOST",
        default_value = "[::]:3031"
    )]
    pub grpc_host: SocketAddr,

    /// The runners authorization token
    #[structopt(long = "grpc-token", env = "WEBALERT_GRPC_TOKEN", required = true)]
    pub token: String,
}
