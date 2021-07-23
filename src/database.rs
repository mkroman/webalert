use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    Error,
};

/// The database pool type. We're using Postgres for now.
pub type DbPool = PgPool;

pub async fn connect(url: &str) -> Result<PgPool, Error> {
    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;

    Ok(pool)
}
