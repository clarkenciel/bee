use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

pub(crate) struct Error {
    status_code: StatusCode,
    message: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (
            self.status_code,
            [("content-type", "application/json")],
            Json(json!({
                "message": self.message,
            })),
        )
            .into_response()
    }
}

impl Error {
    pub(crate) fn new(status_code: StatusCode, message: String) -> Self {
        Self {
            status_code,
            message,
        }
    }
}
