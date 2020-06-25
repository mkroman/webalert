use log::debug;
use tokio_postgres::{Client, Config, Error as PostgresError};

pub fn init() {}

/// Creates a new postgres client config from CLI options
pub fn postgres_config_from_opts(
    opts: crate::cli::Opts,
) -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = Config::new();

    config
        .user(&opts.postgres_user)
        .password(&opts.postgres_password)
        .dbname(&opts.postgres_db)
        .host(&opts.postgres_host);

    Ok(config)
}

pub async fn connect(config: Config) -> Result<Client, PostgresError> {
    let (client, conn) = config.connect(tokio_postgres::tls::NoTls).await?;

    tokio::spawn(async move {
        debug!("Connected to postgres");

        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}
