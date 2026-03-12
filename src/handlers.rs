use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json, Extension,
};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use url::