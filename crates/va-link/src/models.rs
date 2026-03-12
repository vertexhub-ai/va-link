use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Link {
    pub id: Uuid,
    pub short_code: String,
    pub original_url: String,
    pub tenant_id: String,
    pub api_key_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub click_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateLink {
    pub original_url: String,
    // Optional expiration date for the short link
    // If not provided, the link might not expire or have a default expiration set by business logic.
    pub expires_at: Option<DateTime<Utc>>,
}
