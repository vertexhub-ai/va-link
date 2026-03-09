use anyhow::Context;
use axum::{
    Router,
    extract::{Json, Path, State},
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use http::StatusCode;
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Debug, Error)]
enum AppError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Link not found")]
    NotFound,
    #[error("Failed to generate short code")]
    ShortCodeGeneration,
    #[error("Internal server error")]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::Sqlx(e) => {
                tracing::error!("SQLX Error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Link not found".to_string()),
            AppError::ShortCodeGeneration => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate short code".to_string(),
            ),
            AppError::Anyhow(e) => {
                tracing::error!("Anyhow Error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };
        (status, Json(serde_json::json!({ "error": error_message }))).into_response()
    }
}

#[derive(Debug, FromRow, Serialize)]
struct Link {
    id: Uuid,
    short_code: String,
    original_url: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateLinkPayload {
    original_url: String,
}

#[derive(Debug, Serialize)]
struct CreateLinkResponse {
    short_code: String,
    original_url: String,
}

async fn create_link(
    State(state): State<AppState>,
    Json(payload): Json<CreateLinkPayload>,
) -> Result<Json<CreateLinkResponse>, AppError> {
    let short_code: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    let link = sqlx::query_as!(
        Link,
        r#"
        INSERT INTO links (short_code, original_url)
        VALUES ($1, $2)
        RETURNING id, short_code, original_url, created_at
        "#,
        short_code,
        payload.original_url
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(CreateLinkResponse {
        short_code: link.short_code,
        original_url: link.original_url,
    }))
}

async fn redirect_link(
    State(state): State<AppState>,
    Path(short_code): Path<String>,
) -> Result<Redirect, AppError> {
    let link = sqlx::query_as!(
        Link,
        "SELECT id, short_code, original_url, created_at FROM links WHERE short_code = $1",
        short_code
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Redirect::to(&link.original_url))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().json().init();

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPool::connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    let app_state = AppState { pool };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/links", post(create_link))
        .route("/:short_code", get(redirect_link))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
