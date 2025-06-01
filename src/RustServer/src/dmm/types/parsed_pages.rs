use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ParsedPages {
    pub page: String,
    pub entry_count: i32,
}