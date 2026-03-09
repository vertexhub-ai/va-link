use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::models::{CreateLink, Link};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

pub async fn create_link(
    State(state): State<AppState>,
    Json(payload): Json<CreateLink>,
) -> impl IntoResponse {
    let new_link = Link {
        id: Uuid::new_v4(),
        original_url: payload.original_url,
        short_code: payload.short_code,
        created_at: chrono::Utc::now(),
    };

    match sqlx::query_as!(
        Link,
        r#"
        INSERT INTO links (id, original_url, short_code, created_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, original_url, short_code, created_at
        "#,
        new_link.id,
        new_link.original_url,
        new_link.short_code,
        new_link.created_at
    )
    .fetch_one(&state.pool)
    .await
    {
        Ok(link) => (StatusCode::CREATED, Json(link)).into_response(),
        Err(e) => {
            error!("Failed to create link: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create link").into_response()
        }
    }
}

pub async fn get_link(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match sqlx::query_as!(
        Link,
        r#"
        SELECT id, original_url, short_code, created_at
        FROM links
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(link)) => (StatusCode::OK, Json(link)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Link not found").into_response(),
        Err(e) => {
            error!("Failed to retrieve link: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to retrieve link").into_response()
        }
    }
}

pub async fn list_links(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query_as!(
        Link,
        r#"
        SELECT id, original_url, short_code, created_at
        FROM links
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.pool)
    .await
    {
        Ok(links) => (StatusCode::OK, Json(links)).into_response(),
        Err(e) => {
            error!("Failed to list links: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to list links").into_response()
        }
    }
}
