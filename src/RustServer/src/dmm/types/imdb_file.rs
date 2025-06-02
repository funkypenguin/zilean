use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImdbFile {
    #[serde(rename = "imdb_id")]
    pub imdb_id: String,

    pub category: Option<String>,
    pub title: Option<String>,
    pub adult: bool,
    pub year: i32,
}
