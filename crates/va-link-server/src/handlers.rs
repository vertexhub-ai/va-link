use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
};
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::{CreateLinkRequest, LinkResponse, UpdateLinkRequest};

pub async fn health() -> &'static str {
    "ok"
}

pub async fn create_link(
    State(pool): State<PgPool>,
    Json(req): Json<CreateLinkRequest>,
) -> Result<(StatusCode, Json<LinkResponse>), AppError> {
    if req.slug.is_empty() {
        return Err(AppError::BadRequest("slug must not be empty".to_string()));
    }
    if req.target_url.is_empty() {
        return Err(AppError::BadRequest(
            "target_url must not be empty".to_string(),
        ));
    }

    let link = sqlx::query_as!(
        crate::models::Link,
        r#"
        INSERT INTO links (slug, target_url, title, click_count, active, created_at, updated_at)
        VALUES ($1, $2, $3, 0, true, NOW(), NOW())
        ON CONFLICT (slug) DO NOTHING
        RETURNING id, slug, target_url, title, click_count, active, created_at, updated_at
        "#,
        req.slug,
        req.target_url,
        req.title,
    )
    .fetch_optional(&pool)
    .await?;

    match link {
        Some(l) => Ok((StatusCode::CREATED, Json(LinkResponse::from(l)))),
        None => Err(AppError::Conflict(format!(
            "slug '{}' already exists",
            req.slug
        ))),
    }
}

pub async fn get_link(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<Json<LinkResponse>, AppError> {
    let link = sqlx::query_as!(
        crate::models::Link,
        r#"
        SELECT id, slug, target_url, title, click_count, active, created_at, updated_at
        FROM links
        WHERE slug = $1
        "#,
        slug,
    )
    .fetch_optional(&pool)
    .await?;

    match link {
        Some(l) => Ok(Json(LinkResponse::from(l))),
        None => Err(AppError::NotFound),
    }
}

pub async fn update_link(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
    Json(req): Json<UpdateLinkRequest>,
) -> Result<Json<LinkResponse>, AppError> {
    let link = sqlx::query_as!(
        crate::models::Link,
        r#"
        UPDATE links
        SET
            target_url = COALESCE($2, target_url),
            title      = COALESCE($3, title),
            active     = COALESCE($4, active),
            updated_at = NOW()
        WHERE slug = $1
        RETURNING id, slug, target_url, title, click_count, active, created_at, updated_at
        "#,
        slug,
        req.target_url,
        req.title,
        req.active,
    )
    .fetch_optional(&pool)
    .await?;

    match link {
        Some(l) => Ok(Json(LinkResponse::from(l))),
        None => Err(AppError::NotFound),
    }
}

pub async fn delete_link(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query!("DELETE FROM links WHERE slug = $1", slug,)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn redirect_link(
    State(pool): State<PgPool>,
    Path(slug): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let link = sqlx::query_as!(
        crate::models::Link,
        r#"
        UPDATE links
        SET click_count = click_count + 1, updated_at = NOW()
        WHERE slug = $1 AND active = true
        RETURNING id, slug, target_url, title, click_count, active, created_at, updated_at
        "#,
        slug,
    )
    .fetch_optional(&pool)
    .await?;

    match link {
        Some(l) => Ok(Redirect::temporary(&l.target_url)),
        None => Err(AppError::NotFound),
    }
}

pub async fn list_links(State(pool): State<PgPool>) -> Result<Json<Vec<LinkResponse>>, AppError> {
    let links = sqlx::query_as!(
        crate::models::Link,
        r#"
        SELECT id, slug, target_url, title, click_count, active, created_at, updated_at
        FROM links
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(links.into_iter().map(LinkResponse::from).collect()))
}
