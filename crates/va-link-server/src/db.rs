use anyhow::{Context, Result};
use sqlx::PgPool;
use tracing::info;

pub async fn setup_database(database_url: &str) -> Result<PgPool> {
    info!("Connecting to database...");
    let pool = PgPool::connect(database_url)
        .await
        .context("Failed to connect to Postgres")?;

    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run database migrations")?;

    info!("Database setup complete.");
    Ok(pool)
}
