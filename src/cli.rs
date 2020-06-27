use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum Opts {
    /// Run the webalert daemon
    #[structopt(alias = "s")]
    Server(ServerOpts),

    /// Perform database operations
    #[structopt(alias = "db")]
    Database,
}

#[derive(StructOpt, Debug)]
pub struct ServerOpts {
    /// The number of webdrivers to run in parallel
    #[structopt(short = "n", default_value = "3")]
    pub num_webdrivers: u64,

    /// PostgreSQL host
    #[structopt(long, default_value = "localhost", env = "postgres_host")]
    pub postgres_host: String,

    /// PostgreSQL user
    #[structopt(long, default_value = "webalert", env = "postgres_user")]
    pub postgres_user: String,

    /// PostgreSQL user password
    #[structopt(long, long, env = "POSTGRES_PASSWORD")]
    pub postgres_password: String,

    /// PostgreSQL database name
    #[structopt(long, default_value = "webalert_development", env = "POSTGRES_DB")]
    pub postgres_db: String,
}
