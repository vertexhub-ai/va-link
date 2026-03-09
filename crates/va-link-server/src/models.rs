use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct Link {
    pub id: uuid::Uuid,
    pub original_url: String,
    pub short_code: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateLink {
    pub original_url: String,
    pub short_code: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_link_struct() {
        let create_link = CreateLink {
            original_url: "https://example.com".to_string(),
            short_code: "abc".to_string(),
        };
        assert_eq!(create_link.original_url,