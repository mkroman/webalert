use tokio_postgres::Error as PostgresError;

/// Error indicating there was a problem during migration
pub enum MigrationError {
    /// An error occurred when talking to Postgres
    PostgresError(PostgresError),
}
