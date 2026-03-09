use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Link {
    pub id: i64,
    pub slug: String,
    pub target_url: String,
    pub title: Option<String>,
    pub click_count: i64,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLinkRequest {
    pub slug: String,
    pub target_url: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLinkRequest {
    pub target_url: Option<String>,
    pub title: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkResponse {
    pub id: i64,
    pub slug: String,
    pub target_url: String,
    pub title: Option<String>,
    pub click_count: i64,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Link> for LinkResponse {
    fn from(l: Link) -> Self {
        Self {
            id: l.id,
            slug: l.slug,
            target_url: l.target_url,
            title: l.title,
            click_count: l.click_count,
            active: l.active,
            created_at: l.created_at,
            updated_at: l.updated_at,
        }
    }
}
