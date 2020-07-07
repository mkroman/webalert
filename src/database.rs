use crate::cli;

use log::debug;
use tokio_postgres::{self as postgres, Client, Config, Error as PostgresError};

/// Connects to the database specified in the CLI `opts` and ten returns the Postgres client
/// instance
pub async fn init(opts: &cli::Opts) -> Result<Client, Box<dyn std::error::Error>> {
    let conf = postgres_config_from_server_opts(&opts)?;
    let conn = connect(conf).await?;

    Ok(conn)
}

/// Sets up the schema migration using the given postgres `conn` and returns the current migration
/// version, if any
pub async fn init_migration(conn: &mut Client) -> Result<(), PostgresError> {
    // Create migration table if it doesn't exist
    prepare_migration(&conn).await?;

    debug!(
        "Current migration version: {}",
        get_migration_version(&conn)
            .await?
            .unwrap_or_else(|| "none".to_owned())
    );

    Ok(())
}

/// Prepares the database by creating migration tables if they don't already exist
///
/// Returns the number of rows modified
pub async fn prepare_migration(db: &Client) -> Result<u64, PostgresError> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            filename VARCHAR(255) NOT NULL PRIMARY KEY
        )",
        &[],
    )
    .await
}

/// Returns the latest migration filename applied to the database, as a string
pub async fn get_migration_version(db: &Client) -> Result<Option<String>, PostgresError> {
    let row = db
        .query_one(
            "SELECT filename FROM schema_migrations ORDER BY filename DESC LIMIT 1",
            &[],
        )
        .await;

    match row {
        Ok(row) => Ok(row.get(0)),
        Err(_) => Ok(None),
    }
}

/// Creates a new postgres client config from CLI options
pub fn postgres_config_from_server_opts(
    opts: &cli::Opts,
) -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = Config::new();

    config
        .user(&opts.postgres_user)
        .password(&opts.postgres_password)
        .dbname(&opts.postgres_db)
        .host(&opts.postgres_host);

    Ok(config)
}

/// Formats a postgres `Host` as a readable string
fn format_postgres_host(host: &postgres::config::Host) -> String {
    match host {
        postgres::config::Host::Tcp(ref s) => format!("tcp://{}", s),
        postgres::config::Host::Unix(ref path) => {
            format!("unix://{}", path.as_path().to_string_lossy())
        }
    }
}

pub async fn connect(config: Config) -> Result<Client, PostgresError> {
    debug!(
        "Connecting to Postgres at {}, using the database `{}'",
        config
            .get_hosts()
            .iter()
            .map(format_postgres_host)
            .collect::<Vec<String>>()
            .join(", "),
        config.get_dbname().unwrap_or("undefined")
    );

    let (client, conn) = config.connect(postgres::tls::NoTls).await?;

    tokio::spawn(async move {
        debug!("Connected to Postgres");

        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }

        debug!("Connection terminated");
    });

    Ok(client)
}
