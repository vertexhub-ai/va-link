use axum::{
    Extension, Router,
    routing::{get, post},
};
use sqlx::PgPool;

use crate::auth::AuthData;
use crate::handlers::{create_link, get_link_details, redirect_link};

pub fn app_router(pool: PgPool) -> Router {
    Router::new()
        .route("/links", post(create_link))
        .route("/links/:short_code", get(get_link_details))
        .route("/:short_code", get(redirect_link))
        .layer(Extension(pool))
        .layer(axum::middleware::from_fn(AuthData::from_request)) // Apply auth middleware
}
