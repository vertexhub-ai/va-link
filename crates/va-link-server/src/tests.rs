use crate::{app, config::AppConfig, db};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt; // for `collect`
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::{str::FromStr, time::Duration};
use tokio::net::TcpListener;
use tower::ServiceExt; // for `call`, `oneshot`

async fn setup_test_environment() -> (PgPool, AppConfig) {
    // Use a unique database for each test run to prevent conflicts
    let db_name = format!("test_va_link_{}", uuid::Uuid::new_v4().simple());
    let admin_options = PgConnectOptions::from_str(
        "postgres://postgres:password@localhost:5432/postgres",
    )
    .expect("Failed to parse admin DB URL");

    let mut conn = sqlx::PgConnection::connect_with(&admin_options)
        .await
        .expect("Failed to connect to admin DB");

    sqlx::query(&format!("DROP DATABASE IF EXISTS {}", db_name))
        .execute(&mut conn)
        .await
        .expect("Failed to drop test database if exists");
    sqlx::query(&format!("CREATE DATABASE {}", db_name))
        .execute(&mut conn)
        .await
        .expect("Failed to create test database");

    let test_db_url = format!("postgres://postgres:password@localhost:5432/{}", db_name);

    let config = AppConfig {
        database_url: test_db_url.clone(),
        server_address: "127.0.0.1:0".to_string(), // Use port 0 for ephemeral port
        short_code_length: 7,
        base_url: "http://localhost:8080".to_string(),
    };

    let pool = PgPoolOptions::new()
        .max_connections(1) // Use a single connection for tests
        .connect(&test_db_url)
        .await
        .expect("Failed to connect to test database");

    db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    (pool, config)
}

#[tokio::test]
async fn test_health_check() {
    let (pool, config) = setup_test_environment().await;
    let app = app(pool, config);

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_and_redirect_link() {
    let (pool, config) = setup_test_environment().await;
    let app = app(pool.clone(), config.clone());

    // 1. Create a short link
    let create_payload = serde_json::json!({
        "original_url": "https://example.com/very/long/path/to/resource",
        "custom_short_code": "mytestlink"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/links")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(create_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let link_response: crate::models::ShortLinkResponse =
        serde_json::from_slice(&body).unwrap();

    assert_eq!(link_response.original_url, "https://example.com/very/long/path/to/resource");
    assert_eq!(link_response.short_code, "mytestlink");
    assert!(link_response.full_short_url.contains("mytestlink"));

    // 2. Redirect using the short code
    let redirect_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/{}", link_response.short_code))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(redirect_response.status(), StatusCode::FOUND);
    assert_eq!(
        redirect_response.headers()["Location"],
        "https://example.com/very/long/path/to/resource"
    );

    // 3. Get