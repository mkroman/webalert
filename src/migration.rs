use std::collections::BTreeMap;

use crate::migrations;

use log::debug;
use sqlx::PgPool;

/// A migration interface that makes it easier to do transactional migrations to specific versions
pub struct MigrationRunner<'a> {
    pool: &'a mut PgPool,
    /// The current migration version
    current_version: Option<String>,
    /// Initialized binary tree that holds all our migrations
    migrations_up: BTreeMap<&'a str, &'a str>,
    /// Initialized binary tree that holds all our migrations
    migrations_down: BTreeMap<&'a str, &'a str>,
}

impl<'a> MigrationRunner<'a> {
    /// Creates a new `MigrationRunner` using the given postgres `client`
    pub fn new(pool: &mut PgPool, current_version: Option<String>) -> MigrationRunner {
        let (migrations_up, migrations_down) = migrations::init();

        MigrationRunner {
            pool,
            current_version,
            migrations_up,
            migrations_down,
        }
    }

    /// Runs all migrations newer than the current, up to and including the specified
    /// `version`
    ///
    /// If `version` is `None`, then all newer migrations are run
    pub async fn migrate_up_to_version(
        &mut self,
        version: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set the target version to the one specified, otherwise use the most recent
        let target_ver = match version {
            Some(v) => v,
            None => self.migrations_up.keys().max().unwrap(),
        };

        // Create a list of migrations to apply
        let migrations: Vec<(&&str, &&str)> = match &self.current_version {
            Some(current_ver) => self
                .migrations_up
                .iter()
                .filter(|(&key, _)| key > current_ver.as_str() && key <= target_ver)
                .collect(),
            None => self
                .migrations_up
                .iter()
                .filter(|(&key, _)| key <= target_ver)
                .collect(),
        };

        if !migrations.is_empty() {
            for (name, &migration) in migrations {
                let mut tx = self.pool.begin().await?;
                debug!("Applying migration {}", name);

                sqlx::query(migration).execute(&mut tx).await?;
                sqlx::query("INSERT INTO schema_migrations (filename) VALUES ($1::TEXT)")
                    .bind(name)
                    .execute(&mut tx)
                    .await?;

                tx.commit().await?;
            }
        }

        Ok(())
    }
}
