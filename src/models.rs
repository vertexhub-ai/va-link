use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Link {
    pub id: Uuid,
    pub original_url: String,
    pub short_code: String,
    pub tenant_id: String,
    pub api_key: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub original_url: String,
}

#[derive(Debug, Serialize)]
pub struct LinkResponse {
    pub short_code: String,
    pub original_url: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {}
