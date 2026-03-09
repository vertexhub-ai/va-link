use anyhow::Result;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

mod errors;
mod handlers;
mod lib;
mod models;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().json().init();

    let database_url = lib::database_url();
    let pool = lib::create_pool(&database_url).await?;

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/links", get(handlers::list_links))
        .route("/links", post(handlers::create_link))
        .route("/links/:slug", get(handlers::get_link))
        .route("/links/:slug", put(handlers::update_link))
        .route("/links/:slug", delete(handlers::delete_link))
        .route("/r/:slug", get(handlers::redirect_link))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}