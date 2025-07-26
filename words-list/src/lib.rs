use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Words {
    pub words: Vec<Word>,
    pub pagination: Pagination,
}

#[derive(Deserialize, Serialize)]
pub struct Word {
    pub text: String,
    pub cursor: Cursor,
}

#[derive(Deserialize, Serialize)]
pub struct Pagination {
    pub next_page: Option<Cursor>,
    pub prev_page: Option<Cursor>,
}

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct Cursor(pub String);
