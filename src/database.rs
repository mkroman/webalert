use crate::cli;

use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    Error,
};

/// The database pool type. We're using Postgres for now.
pub type DbPool = PgPool;

/// Connects to the database specified in the CLI `opts` and ten returns the Postgres client
/// instance
pub async fn init(opts: &cli::Opts) -> Result<PgPool, Box<dyn std::error::Error>> {
    let conn = connect(&opts.postgres_url).await?;

    Ok(conn)
}

pub async fn connect(url: &str) -> Result<PgPool, Error> {
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;

    Ok(pool)
}
