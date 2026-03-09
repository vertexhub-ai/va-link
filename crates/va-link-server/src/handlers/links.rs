use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AppError;
use crate::error::AppResult;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Link {
    pub id: i64,
    pub slug: String,
    pub target_url: String,
    pub title: Option<String>,
    pub click_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub slug: String,
    pub target_url: String,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLinkRequest {
    pub target_url: Option<String>,
    pub title: Option<String>,
}

pub async fn list_links(State(pool): State<PgPool>) -> AppResult<Json<Vec<Link>>> {
    let links = sqlx::query_as!(
        Link,
        r#"SELECT id, slug, target_url, title, click_count, created_at, updated_at
           FROM links
           ORDER BY created_at DESC"#
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(links))
}

pub async fn get_link(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> AppResult<Json<Link>> {
    let link = sqlx::query_as!(
        Link,
        r#"SELECT id, slug, target_url, title, click_count, created_at, updated_at
           FROM links
           WHERE slug = $1"#,
        slug
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(link))
}

pub async fn create_link(
    State(pool): State<PgPool>,
    Json(req): Json<CreateLinkRequest>,
) -> AppResult<Json<Link>> {
    if req.slug.trim().is_empty() {
        return Err(AppError::BadRequest("slug must not be empty".to_string()));
    }
    if req.target_url.trim().is_empty() {
        return Err(AppError::BadRequest(
            "target_url must not be empty".to_string(),
        ));
    }
    let link = sqlx::query_as!(
        Link,
        r#"INSERT INTO links (slug, target_url, title)
           VALUES ($1, $2, $3)
           RETURNING id, slug, target_url, title, click_count, created_at, updated_at"#,
        req.slug.trim(),
        req.target_url.trim(),
        req.title.as_deref().map(str::trim)
    )
    .fetch_one(&pool)
    .await?;
    Ok(Json(link))
}

pub async fn update_link(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
    Json(req): Json<UpdateLinkRequest>,
) -> AppResult<Json<Link>> {
    let link = sqlx::query_as!(
        Link,
        r#"UPDATE links
           SET target_url  = COALESCE($2, target_url),
               title       = COALESCE($3, title),
               updated_at  = NOW()
           WHERE slug = $1
           RETURNING id, slug, target_url, title, click_count, created_at, updated_at"#,
        slug,
        req.target_url.as_deref().map(str::trim),
        req.title.as_deref().map(str::trim)
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(link))
}

pub async fn delete_link(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> AppResult<axum::http::StatusCode> {
    let result = sqlx::query!(r#"DELETE FROM links WHERE slug = $1"#, slug)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn redirect_link(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> AppResult<axum::response::Redirect> {
    let link = sqlx::query_as!(
        Link,
        r#"UPDATE links
           SET click_count = click_count + 1,
               updated_at  = NOW()
           WHERE slug = $1
           RETURNING id, slug, target_url, title, click_count, created_at, updated_at"#,
        slug
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(axum::response::Redirect::permanent(&link.target_url))
}
