use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use base64::Engine as _;
use serde::Deserialize;

pub(crate) async fn list_words<Service>(
    State(service): State<Service>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse
where
    Service: crate::services::words::ListWords,
{
    let Ok(cursor) = query
        .cursor
        .map(cursor_from_url)
        .unwrap_or_else(|| Ok(Default::default()))
    else {
        return crate::responses::Error::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid cursor".to_owned(),
        )
        .into_response();
    };

    match service.list(&cursor, None).await {
        Err(e) => crate::responses::Error::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            .into_response(),
        Ok(crate::services::words::ListedWords { words, next_page }) => {
            (
                StatusCode::OK,
                [("content-type", "application/json")],
                Json(words_list::Words {
                    words: words
                        .into_iter()
                        .map(|w| words_list::Word {
                            text: w.text,
                            cursor: words_list::Cursor(cursor_to_url(&w.cursor).unwrap()),
                        })
                        .collect(),
                    pagination: words_list::Pagination {
                        next_page: next_page
                            .and_then(|np| cursor_to_url(&np).map(|c| words_list::Cursor(c)).ok()),
                        prev_page: None,
                    },
                }),
            )
        }
        .into_response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct ListQuery {
    cursor: Option<String>,
}

fn cursor_to_url(
    cursor: &crate::services::words::ListCursor,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut output = String::new();
    base64::engine::general_purpose::URL_SAFE.encode_string(cursor.after.as_bytes(), &mut output);
    Ok(output)
}

fn cursor_from_url(
    param: String,
) -> Result<crate::services::words::ListCursor, Box<dyn std::error::Error>> {
    let after = base64::engine::general_purpose::URL_SAFE
        .decode(&param)
        .map_err(Box::new)?;

    let after = String::from_utf8(after).map_err(Box::new)?;
    Ok(crate::services::words::ListCursor { after })
}

pub(crate) async fn search<Service>(
    State(service): State<Service>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse
where
    Service: crate::services::words::SearchWords,
{
    match service.search(&query.query).await {
        Err(e) => crate::responses::Error::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            .into_response(),
        Ok(results) => (
            StatusCode::OK,
            [("content-type", "application/json")],
            Json(search::SearchedWords { words: results }),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct SearchQuery {
    #[serde(alias = "q")]
    query: String,
}
