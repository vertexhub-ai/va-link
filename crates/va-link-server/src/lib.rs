pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;

use axum::{
    routing::{get, post},
    Router,
};
use handlers::{AppStateData, AppState};
use sqlx::PgPool;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub fn app(pool: PgPool, config: config::AppConfig) -> Router {
    let state = AppStateData { pool, config };

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/links", post(handlers::create_short_link))
        .route("/:short_code", get(handlers::redirect_short_link))
        .route("/links/:short_code/stats", get(handlers::get_link_stats))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests;
