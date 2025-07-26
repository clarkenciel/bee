use axum::{
    Router,
    routing::{get, post},
};

use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt};

mod handlers;
mod puzzle_config;
mod responses;
mod services;

#[tokio::main]
async fn main() {
    if let Err(e) = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_env("BEE_LOG_LEVEL"))
        .try_init()
    {
        eprintln!("Failed to init tracing: {}", e);
    }

    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Failed to load dotenv file: {}", e);
    }

    let pool_url = dotenvy::var("DATABASE_URL").expect("Failed to get database url from env");

    let dbpool = sqlx::PgPool::connect(&pool_url)
        .await
        .expect("Failed to connect to postgres instance");
    let index = ServeFile::new("index.html");
    let assets = ServeDir::new("assets");
    let app = Router::new()
        .route(
            "/api/puzzle/daily/config",
            get(handlers::puzzle_config::puzzle_config),
        )
        .with_state(crate::puzzle_config::ConfigProvider::new(dbpool.clone()))
        .route(
            "/api/words",
            post(handlers::words::add_words::<crate::services::words::pg::AddWords>),
        )
        .with_state(crate::services::words::pg::AddWords(dbpool.clone()))
        .route(
            "/api/words/remove",
            post(handlers::words::remove_words::<crate::services::words::pg::RemoveWords>),
        )
        .with_state(crate::services::words::pg::RemoveWords(dbpool.clone()))
        .nest_service("/assets", assets)
        .fallback_service(index);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
