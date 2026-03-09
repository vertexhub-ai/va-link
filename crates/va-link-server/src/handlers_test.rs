#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use crate::models::{CreateLinkRequest, LinkResponse};

    fn make_create_request(slug: &str, url: &str) -> CreateLinkRequest {
        CreateLinkRequest {
            slug: slug.to_string(),
            target_url: url.to_string(),
            title: None,
        }
    }

    #[test]
    fn test_create_link_request_fields() {
        let req = make_create_request("test-slug", "https://example.com");
        assert_eq!(req.slug, "test-slug");
        assert_eq!(req.target_url, "https://example.com");
        assert!(req.title.is_none());
    }

    #[test]
    fn test_create_link_request_with_title() {
        let req = CreateLinkRequest {
            slug: "my-link".to_string(),
            target_url: "https://example.com/path".to_string(),
            title: Some("My Link Title".to_string()),
        };
        assert_eq!(req.slug, "my-link");
        assert_eq!(req.title, Some("My Link Title".to_string()));
    }

    #[test]
    fn test_link_response_from_model() {
        use crate::models::Link;
        let now = chrono::Utc::now();
        let link = Link {
            id: 1,
            slug: "abc".to_string(),
            target_url: "https://example.com".to_string(),
            title: None,
            click_count: 42,
            active: true,
            created_at: now,
            updated_at: now,
        };
        let resp = LinkResponse::from(link);
        assert_eq!(resp.id, 1);
        assert_eq!(resp.slug, "abc");
        assert_eq!(resp.click_count, 42);
        assert!(resp.active);
    }

    #[test]
    fn test_update_link_request_all_none() {
        use crate::models::UpdateLinkRequest;
        let req = UpdateLinkRequest {
            target_url: None,
            title: None,
            active: None,
        };
        assert!(req.target_url.is_none());
        assert!(req.title.is_none());
        assert!(req.active.is_none());
    }

    #[test]
    fn test_update_link_request_partial() {
        use crate::models::UpdateLinkRequest;
        let req = UpdateLinkRequest {
            target_url: Some("https://new-url.com".to_string()),
            title: None,
            active: Some(false),
        };
        assert_eq!(req.target_url, Some("https://new-url.com".to_string()));
        assert!(req.title.is_none());
        assert_eq!(req.active, Some(false));
    }

    #[test]
    fn test_slug_empty_validation() {
        let req = make_create_request("", "https://example.com");
        assert!(req.slug.is_empty());
    }

    #[test]
    fn test_target_url_empty_validation() {
        let req = make_create_request("slug", "");
        assert!(req.target_url.is_empty());
    }

    #[test]
    fn test_link_serialization() {
        use crate::models::Link;
        let now = chrono::Utc::now();
        let link = Link {
            id: 10,
            slug: "serialize-test".to_string(),
            target_url: "https://serialize.example.com".to_string(),
            title: Some("Serialization Test".to_string()),
            click_count: 0,
            active: true,
            created_at: now,
            updated_at: now,
        };
        let json_val = serde_json::to_value(&link).expect("serialization failed");
        assert_eq!(json_val["slug"], "serialize-test");
        assert_eq!(json_val["click_count"], 0);
        assert_eq!(json_val["active"], true);
    }
}
