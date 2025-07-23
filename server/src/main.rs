use axum::{
    routing::get,
    Router,
};

use tower_http::services::{ServeDir,ServeFile};

mod puzzle_config;
mod handlers;

#[tokio::main]
async fn main() {
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Failed to load dotenv file: {}", e);
    }

    let pool_url = dotenvy::var("DATABASE_URL").expect("Failed to get database url from env");

    let dbpool = sqlx::PgPool::connect(&pool_url).await.expect("Failed to connect to postgres instance");
    let index = ServeFile::new("index.html");
    let assets = ServeDir::new("assets");
    let app = Router::new()
        .route("/puzzle/daily/config", get(handlers::puzzle_config))
        .with_state(crate::puzzle_config::ConfigProvider::new(dbpool))
        .nest_service("/assets", assets)
        .fallback_service(index);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


