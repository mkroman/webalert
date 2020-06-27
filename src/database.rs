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

/// Sets up the schema migration using the given postgres `conn`
pub async fn init_migration(conn: &mut Client) -> Result<(), PostgresError> {
    // Create migration table if it doesn't exist
    prepare_migration(&conn).await?;

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

    Ok(row.get(0))
}

pub async fn migrate_up_to(db: &mut Client, version: Option<&str>) -> Result<(), PostgresError> {
    debug!("Migrating database to version `{:?}'", version);

    unimplemented!()
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

pub async fn connect(config: Config) -> Result<Client, PostgresError> {
    debug!(
        "Connecting to Postgres at {:?}, using the database `{}'",
        config.get_hosts(),
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
