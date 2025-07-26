use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SearchedWords {
    pub words: Vec<String>,
}
