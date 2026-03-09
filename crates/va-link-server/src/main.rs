use anyhow::Result;
use axum::Router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use va_link_server::{app, config::AppConfig, db};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "va_link_server=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // Load configuration
    let config = AppConfig::load()?;
    tracing::info!("Configuration loaded: {:?}", config);

    // Initialize database pool and run migrations
    let pool = db::init_db_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;

    // Build Axum app
    let app = app(pool, config.clone());

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.server_address).await?;
    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
