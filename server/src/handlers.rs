use axum::{
    extract::{Query, State},
    http,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::puzzle_config;

pub async fn puzzle_config(
    State(configs): State<puzzle_config::ConfigProvider>,
    Query(query): Query<TimezoneQuery>,
) -> impl IntoResponse {
    let config = configs.get_config(&query.tz.parse().unwrap()).await.unwrap();
    let body = serde_json::to_string(&config).unwrap();
    (
        http::StatusCode::OK,
        [("content-type", "application/json")],
        body,
    )
}

#[derive(Deserialize)]
pub struct TimezoneQuery {
    tz: String,
}
