use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use chrono::{Duration, Utc};
use rand::{Rng, distributions::Alphanumeric};
use sqlx::{PgPool, query, query_as};
use url::Url;
use validator::Validate;

use crate::{
    config::AppConfig,
    error::AppError,
    models::{Click, CreateLinkRequest, LinkStats, ShortLink, ShortLinkResponse},
};

pub type AppState = State<AppStateData>;

#[derive(Clone)]
pub struct AppStateData {
    pub pool: PgPool,
    pub config: AppConfig,
}

pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn create_short_link(
    State(state): AppState,
    Json(mut req): Json<CreateLinkRequest>,
) -> Result<Json<ShortLinkResponse>, AppError> {
    req.validate()?;

    // Validate original_url is a valid URL
    Url::parse(&req.original_url)?;

    let short_code = if let Some(custom_code) = req.custom_short_code {
        // Check if custom code already exists
        let existing_link = query_as!(
            ShortLink,
            "SELECT * FROM short_links WHERE short_code = $1",
            custom_code
        )
        .fetch_optional(&state.pool)
        .await?;

        if existing_link.is_some() {
            return Err(AppError::Conflict);
        }
        custom_code
    } else {
        // Generate a random short code
        generate_unique_short_code(&state.pool, state.config.short_code_length).await?
    };

    let created_at = Utc::now();
    let expires_at = req.expires_at; // Can be None

    let new_link = query_as!(
        ShortLink,
        "INSERT INTO short_links (id, original_url, short_code, created_at, expires_at, clicks)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, original_url, short_code, created_at, expires_at, user_id, clicks",
        Uuid::new_v4(),
        req.original_url,
        short_code,
        created_at,
        expires_at,
        0i64
    )
    .fetch_one(&state.pool)
    .await?;

    let full_short_url = format!("{}/{}", state.config.base_url, new_link.short_code);

    Ok(Json(ShortLinkResponse {
        id: new_link.id,
        original_url: new_link.original_url,
        short_code: new_link.short_code,
        full_short_url,
        created_at: new_link.created_at,
        expires_at: new_link.expires_at,
        clicks: new_link.clicks,
    }))
}

pub async fn redirect_short_link(
    State(state): AppState,
    Path(short_code): Path<String>,
    headers: axum::http::HeaderMap,
) -> Result<Redirect, AppError> {
    let link = query_as!(
        ShortLink,
        "SELECT * FROM short_links WHERE short_code = $1",
        short_code
    )
    .fetch_optional(&state.pool)
    .await?;

    let link = match link {
        Some(l) => l,
        None => return Err(AppError::NotFound),
    };

    // Check if link has expired
    if let Some(expires_at) = link.expires_at {
        if Utc::now() > expires_at {
            return Err(AppError::NotFound); // Or a specific "Expired" error
        }
    }

    // Record click
    let ip_address = headers
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let user_agent = headers
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    query!(
        "INSERT INTO clicks (id, short_link_id, clicked_at, ip_address, user_agent)
         VALUES ($1, $2, $3, $4, $5)",
        Uuid::new_v4(),
        link.id,
        Utc::now(),
        ip_address,
        user_agent
    )
    .execute(&state.pool)
    .await?;

    // Increment click count on short_links table
    query!(
        "UPDATE short_links SET clicks = clicks + 1 WHERE id = $1",
        link.id
    )
    .execute(&state.pool)
    .await?;

    Ok(Redirect::to(&link.original_url))
}

pub async fn get_link_stats(
    State(state): AppState,
    Path(short_code): Path<String>,
) -> Result<Json<LinkStats>, AppError> {
    let short_link = query_as!(
        ShortLink,
        "SELECT * FROM short_links WHERE short_code = $1",
        short_code
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let total_clicks = query!(
        "SELECT COUNT(*) as count FROM clicks WHERE short_link_id = $1",
        short_link.id
    )
    .fetch_one(&state.pool)
    .await?
    .count
    .unwrap_or(0); // COUNT(*) returns BIGINT, so it's i64

    let last_click_at = query!(
        "SELECT MAX(clicked_at) as last_click FROM clicks WHERE short_link_id = $1",
        short_link.id
    )
    .fetch_one(&state.pool)
    .await?
    .last_click;

    Ok(Json(LinkStats {
        short_link,
        total_clicks,
        last_click_at,
    }))
}

async fn generate_unique_short_code(pool: &PgPool, length: usize) -> Result<String, AppError> {
    loop {
        let code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();

        let exists = query!(
            "SELECT short_code FROM short_links WHERE short_code = $1",
            code
        )
        .fetch_optional(pool)
        .await?;

        if exists.is_none() {
            return Ok(code);
        }
    }
}
