use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use crate::services::words::AddWords;

pub(crate) async fn add_words<Service>(
    State(service): State<Service>,
    Json(form): Json<AddWordsForm>,
) -> impl IntoResponse
where
    Service: AddWords,
{
    if form.words.iter().any(|w| w.len() < 4 || !w.is_ascii()) {
        return crate::responses::Error::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid words detected. Words must be >= 4 ascii alphabetic characters long."
                .to_owned(),
        )
        .into_response();
    }

    match service
        .add_words(form.words.into_iter().map(|s| s.to_lowercase()).collect())
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => crate::responses::Error::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct AddWordsForm {
    pub(crate) words: Vec<String>,
}
