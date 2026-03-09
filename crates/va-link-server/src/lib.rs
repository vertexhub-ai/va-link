use anyhow::Result;
use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    Ok(pool)
}

pub fn database_url() -> String {
    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

#[cfg(test)]
pub fn test_database_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/va_link_test".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_url_fallback() {
        if std::env::var("TEST_DATABASE_URL").is_err() {
            let url = test_database_url();
            assert!(url.contains("va_link_test"));
        }
    }
}
