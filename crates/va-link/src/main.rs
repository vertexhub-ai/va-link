use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use dotenvy::dotenv;
use sqlx::{PgPool, Pool, Postgres};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[get("/health")]
async fn health_check(db_pool: web::Data<Pool<Postgres>>) -> impl Responder {
    match sqlx::query("SELECT 1").execute(db_pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().body("va-link is healthy and connected to DB!"),
        Err(e) => {
            tracing::error!("Database connection failed: {:?}", e);
            HttpResponse::ServiceUnavailable().body(format!("DB connection failed: {}", e))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres.");

    tracing::info!("Starting va-link service on 127.0.0.1:8080...");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone())) // Pass database pool to application data
            .service(health_check)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
