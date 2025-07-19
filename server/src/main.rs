use axum::{
    routing::get,
    Router,
};

mod puzzle_config;
mod handlers;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/puzzle/daily/config", get(handlers::puzzle_config))
        .with_state(crate::puzzle_config::ConfigProvider::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


