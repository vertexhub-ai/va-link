use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use dotenvy::dotenv;
use std::env;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{WithExportConfig, new_pipeline};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use futures::FutureExt;

mod handlers;
mod models;
mod lib; // For API key hashing

pub struct AppState {
    pub db_pool: PgPool,
    pub hmac_secret: Vec<u8>,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    // Initialize OpenTelemetry
    init_otel_tracer()?;

    info!("Starting va-link service...");

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let hmac_secret_key = env::var("HMAC_SECRET_KEY")
        .expect("HMAC_SECRET_KEY must be set");
    let server_address = env::var("SERVER_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    let pool = PgPool::connect(&database_url).await?;
    info!("Database connection established.");

    // Run migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| {
            error!("Failed to run migrations: {:?}", e);
            e
        })?;
    info!("Database migrations completed.");

    let hmac_secret = hmac_secret_key.as_bytes().to_vec();

    let app_state = web::Data::new(AppState {
        db_pool: pool.clone(),
        hmac_secret,
    });

    info!("Server starting at: {}", server_address);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone()) // AppState for handlers
            .service(web::resource("/health").to(handlers::health_check))
            .service(web::resource("/link").route(web::post().to(handlers::create_link)))
            .service(web::resource("/{short_code}").route(web::get().to(handlers::redirect_link)))
    })
    .bind(&server_address)?
    .run()
    .await?;

    // Shutdown OpenTelemetry tracer
    global::shutdown_tracer_provider();

    Ok(())
}

fn init_otel_tracer() -> anyhow::Result<()> {
    let collector_url = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let tracer = new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(collector_url),
        )
        .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", "va-link"),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    global::set_tracer_provider(tracer.provider().expect("Failed to get tracer provider"));

    Ok(())
}