use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn test_pool() -> sqlx::PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/va_link_test".to_string());
    let pool = sqlx::PgPool::connect(&url).await.expect("connect");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrate");
    sqlx::query!("DELETE FROM links")
        .execute(&pool)
        .await
        .expect("clean");
    pool
}

#[tokio::test]
async fn test_health() {
    let pool = test_pool().await;
    let app = va_link_server::router::build(pool);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_and_get_link() {
    let pool = test_pool().await;
    let app = va_link_server::router::build(pool);

    let body = serde_json::to_vec(&json!({
        "slug": "test-slug",
        "target_url": "https://example.com",
        "title": "Example"
    }))
    .unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/links")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let val: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(val["slug"], "test-slug");
    assert_eq!(val["click_count"], 0);

    let resp2 = app
        .oneshot(
            Request::builder()
                .uri("/api/links/test-slug")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp2.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_not_found() {
    let pool = test_pool().await;
    let app = va_link_server::router::build(pool);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/links/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_link_empty_slug() {
    let pool = test_pool().await;
    let app = va_link_server::router::build(pool);
    let body =
        serde_json::to_vec(&json!({"slug": "  ", "target_url": "https://example.com"})).unwrap();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/links")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
