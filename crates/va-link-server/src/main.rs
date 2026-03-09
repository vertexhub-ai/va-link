use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use sqlx::PgPool;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

mod lib; // Import the lib module

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "va_link_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/va_link".to_string());

    let pool = lib::setup_database(&database_url).await?;

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/links", post(create_link))
        .route("/:short_code", get(redirect_link))
        .with_state(pool); // Add the database pool as state

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn create_link(
    State(pool): State<PgPool>,
    Json(payload): Json<lib::CreateLinkPayload>,
) -> impl IntoResponse {
    let short_code = generate_short_code(); // Generate a unique short code

    match sqlx::query_as!(
        lib::Link,
        r#"
        INSERT INTO links (short_code, original_url)
        VALUES ($1, $2)
        RETURNING id, short_code, original_url, created_at
        "#,
        short_code,
        payload.original_url
    )
    .fetch_one(&pool)
    .await
    {
        Ok(link) => (StatusCode::CREATED, Json(link)).into_response(),
        Err(e) => {
            tracing::error!("Failed to create link: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create link"})),
            )
                .into_response()
        }
    }
}

async fn redirect_link(Path(short_code): Path<String>, State(pool): State<PgPool>) -> impl IntoResponse {
    match sqlx::query_as!(
        lib::Link,
        r#"
        SELECT id, short_code, original_url, created_at
        FROM links
        WHERE short_code = $1
        "#,
        short_code
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(link)) => Redirect::temporary(&link.original_url).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Link not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to retrieve link: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve link".to_string(),
            )
                .into_response()
        }
    }
}

fn generate_short_code() -> String {
    // A simple way to generate a short code. In a real application,
    // you might want to ensure uniqueness and handle collisions.
    Uuid::new_v4().to_string().split_at(8).0.to_string()
}