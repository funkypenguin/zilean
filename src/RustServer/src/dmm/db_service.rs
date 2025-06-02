#![allow(dead_code)]

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use crate::dmm::types::dmm_last_import::DmmLastImport;
use crate::dmm::types::parsed_pages::ParsedPages;

#[async_trait]
pub trait DmmDbService: Send + Sync {
    async fn get_dmm_last_import(&self) -> Result<Option<DmmLastImport>>;
    async fn set_dmm_import(&self, import: &DmmLastImport) -> Result<()>;
    async fn add_pages_to_ingested(&self, pages: &[ParsedPages]) -> Result<()>;
    async fn add_page_to_ingested(&self, page: &ParsedPages) -> Result<()>;
    async fn get_ingested_pages(&self) -> Result<Vec<ParsedPages>>;
}

pub struct PgDmmDbService {
    pool: PgPool,
}

impl PgDmmDbService {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl DmmDbService for PgDmmDbService {
    async fn get_dmm_last_import(&self) -> Result<Option<DmmLastImport>> {
        let row: Option<(serde_json::Value,)> = sqlx::query_as(
            "SELECT \"Value\" FROM \"ImportMetadata\" WHERE \"Key\" = 'DmmLastImport'"
        )
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.and_then(|(val,)| serde_json::from_value(val).ok()))
    }

    async fn set_dmm_import(&self, import: &DmmLastImport) -> Result<()> {
        let val = serde_json::to_value(import)?;
        sqlx::query(
            r#"
            INSERT INTO "ImportMetadata" ("Key", "Value")
            VALUES ('DmmLastImport', $1)
            ON CONFLICT (key) DO UPDATE SET "Value" = EXCLUDED."Value"
            "#
        )
            .bind(val)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    async fn add_pages_to_ingested(&self, pages: &[ParsedPages]) -> Result<()> {
        for page in pages {
            self.add_page_to_ingested(page).await?;
        }
        Ok(())
    }

    async fn add_page_to_ingested(&self, page: &ParsedPages) -> Result<()> {
        sqlx::query(
            "INSERT INTO \"ParsedPages\" (\"Page\", \"EntryCount\") VALUES ($1, $2)"
        )
            .bind(&page.page)
            .bind(page.entry_count as i32)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_ingested_pages(&self) -> Result<Vec<ParsedPages>> {
        let rows = sqlx::query("SELECT \"Page\", \"EntryCount\" FROM \"ParsedPages\"")
            .map(|row: sqlx::postgres::PgRow| ParsedPages {
                page: row.get("Page"),
                entry_count: row.get("EntryCount"),
            })
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }
}