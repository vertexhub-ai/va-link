use anyhow::Result;
use sqlx::{
    PgPool,
    migrate::Migrator,
    postgres::{PgPoolOptions, PgRow},
};

pub type DbPool = PgPool;

pub async fn init_db_pool(database_url: &str) -> Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &DbPool) -> Result<()> {
    // This path is relative to the Cargo.toml of the crate
    let migrator = Migrator::new(std::path::Path::new("./migrations")).await?;
    migrator.run(pool).await?;
    tracing::info!("Database migrations applied successfully.");
    Ok(())
}

// Re-export PgRow for convenience in models/handlers if needed
pub use PgRow;
