use crate::lib::hash_api_key;
use crate::main::AppState; // Import AppState from main
use crate::models::{CreateLink, Link};
use actix_web::{HttpResponse, Responder, http::header, web};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use sqlx::{PgPool, query, query_as};
use tracing::{error, info};
use uuid::Uuid;

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
