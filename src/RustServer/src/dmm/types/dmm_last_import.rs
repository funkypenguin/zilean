use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DmmLastImport {
    pub occured_at: time::OffsetDateTime,
}
