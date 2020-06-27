use log::debug;
use tokio_postgres::{Client, Config, Error as PostgresError};

/// Initializes the database by creating a table for migrations
pub async fn init(conn: &Client) -> Result<(), PostgresError> {
    prepare_migration(conn).await?;

    debug!(
        "Current migration version: {}",
        get_migration_version(&conn).await?
    );

    Ok(())
}

/// Prepares the database by creating migration tables if they don't already exist
///
/// Returns the number of rows modified
pub async fn prepare_migration(db: &Client) -> Result<u64, PostgresError> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            filename TEXT NOT NULL
        )",
        &[],
    )
    .await
}

/// Returns the latest migration filename applied to the database
pub async fn get_migration_version(db: &Client) -> Result<String, PostgresError> {
    let row = db
        .query_one(
            "SELECT filename FROM schema_migrations ORDER BY filename DESC LIMIT 1",
            &[],
        )
        .await?;

    Ok(row.get("filename"))
}

/// Creates a new postgres client config from CLI options
pub fn postgres_config_from_server_opts(
    opts: crate::cli::ServerOpts,
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
    debug!(
        "Connecting to Postgres at {:?}, using the database `{}'",
        config.get_hosts(),
        config.get_dbname().unwrap_or("undefined")
    );

    let (client, conn) = config.connect(tokio_postgres::tls::NoTls).await?;

    tokio::spawn(async move {
        debug!("Connected to Postgres");

        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }

        debug!("Connection terminated");
    });

    Ok(client)
}
