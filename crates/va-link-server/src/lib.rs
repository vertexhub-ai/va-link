use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Link {
    pub id: Uuid,
    pub short_code: String,
    pub original_url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateLinkPayload {
    pub original_url: String,
}

pub async fn setup_database(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Executor;

    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("DATABASE_URL_TEST")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/va_link_test".to_string());
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        // Ensure a clean state for each test
        pool.execute("DROP TABLE IF EXISTS links").await.expect("Failed to drop table");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations for test db");
        pool
    }

    #[tokio::test]
    async fn test_create_and_get_link() {
        let pool = setup_test_db().await;

        let original_url = "https://example.com/long/path/to/resource";
        let short_code = Uuid::new_v4().to_string().split_at(8).0.to_string(); // Simple short code generation for test

        let new_link