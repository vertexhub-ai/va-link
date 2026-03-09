use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ShortLink {
    pub id: Uuid,
    pub original_url: String,
    pub short_code: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>, // Optional, for authenticated users
    pub clicks: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Click {
    pub id: Uuid,
    pub short_link_id: Uuid,
    pub clicked_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateLinkRequest {
    #[validate(url(message = "Original URL must be a valid URL"))]
    pub original_url: String,
    #[validate(
        length(
            min = 4,
            max = 20,
            message = "Custom short code must be between 4 and 20 characters long"
        ),
        alphanumeric(message = "Custom short code must be alphanumeric")
    )]
    pub custom_short_code: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkStats {
    pub short_link: ShortLink,
    pub total_clicks: i64,
    pub last_click_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortLinkResponse {
    pub id: Uuid,
    pub original_url: String,
    pub short_code: String,
    pub full_short_url: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub clicks: i64,
}

impl From<ShortLink> for ShortLinkResponse {
    fn from(link: ShortLink) -> Self {
        unimplemented!()
    } // This will be implemented in handlers
}
