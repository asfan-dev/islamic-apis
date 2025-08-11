use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tracing::info;

use crate::{config::DatabaseConfig, error::ApiResult};

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(config: &DatabaseConfig) -> ApiResult<Self> {
        info!("Connecting to database...");

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout))
            .idle_timeout(Duration::from_secs(config.idle_timeout))
            .connect(&config.url)
            .await?;

        info!("Database connected successfully");
        Ok(Database { pool })
    }

    pub async fn health_check(&self) -> ApiResult<()> {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }

    pub async fn run_migrations(&self, migrations_path: &str) -> ApiResult<()> {
        info!("Running database migrations from: {}", migrations_path);

        // Note: In production, you might want to use sqlx-cli instead
        // For now, this is a placeholder for manual migration handling

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        // This test requires a running PostgreSQL instance
        // Skip if DATABASE_URL is not set
        if std::env::var("DATABASE_URL").is_err() {
            return;
        }

        let config = DatabaseConfig {
            url: std::env::var("DATABASE_URL").unwrap(),
            max_connections: 5,
            min_connections: 1,
            connect_timeout: 30,
            idle_timeout: 600,
        };

        let db = Database::new(&config).await;
        assert!(db.is_ok());

        if let Ok(db) = db {
            assert!(db.health_check().await.is_ok());
        }
    }
}
