mod models;
mod db;
mod handlers;
mod routes;
mod error;
mod auth;

use axum::{
    Router,
};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    setup_tracing();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::init_db_pool(&database_url).await?;
    db::run_migrations(&pool).await?;

    let app = routes::app_router(pool);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse()?));

    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn setup_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "va_link=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt; // for `call`, `ready`

    // Helper to create a test app (without a real DB connection for simplicity in this diff)
    async fn setup_test_app() -> Router {
        // In a real integration test, you would connect to a test database.
        // For this diff, we'll create a dummy pool to satisfy the type system.
        // This test will only cover middleware logic that doesn't hit the DB.
        let pool = PgPool::connect("postgres://user:password@localhost/test_db").await.unwrap();
        routes::app_router(pool)
    }

    #[tokio::test]
    async fn test_create_link_missing_auth() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/links")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"original_url": "https://example.com"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert!(String::from_utf8_lossy(&body).contains("Missing Tenant-Id or Api-Key headers"));
    }

    #[tokio::test]
    async fn test_generate_short_code_length() {
        let code = crate::handlers::generate_short_code();
        assert_eq!(code.len(), 6);
    }

    #[tokio::test]
    async fn test_generate_short_code_alphanumeric() {
        let code = crate::handlers::generate_short_code();
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}