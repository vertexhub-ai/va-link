use anyhow::Result;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub server_address: String,
    pub short_code_length: usize,
    pub base_url: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        dotenv().ok(); // Load .env file if it exists

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/va_link".to_string());
        let server_address =
            env::var("SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let short_code_length = env::var("SHORT_CODE_LENGTH")
            .unwrap_or_else(|_| "7".to_string())
            .parse::<usize>()
            .expect("SHORT_CODE_LENGTH must be a valid integer");
        let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

        Ok(AppConfig {
            database_url,
            server_address,
            short_code_length,
            base_url,
        })
    }
}
