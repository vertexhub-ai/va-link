use axum::{
    Router,
    routing::{delete, get, post, put},
};
use sqlx::PgPool;

use crate::handlers::links;

pub fn build(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route(
            "/api/links",
            get(links::list_links).post(links::create_link),
        )
        .route(
            "/api/links/:slug",
            get(links::get_link)
                .put(links::update_link)
                .delete(links::delete_link),
        )
        .route("/r/:slug", get(links::redirect_link))
        .with_state(pool)
}
