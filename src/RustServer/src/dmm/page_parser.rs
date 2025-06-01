#![allow(dead_code)]

use std::path::Path;
use std::sync::Arc;
use lazy_static::lazy_static;
use regex::Regex;
use dashmap::DashMap;
use serde_json::Value;
use tokio::fs::{read_dir, read_to_string};
use tracing::{error, info};
use crate::dmm::dmm_service::DmmService;
use crate::dmm::types::extracted_dmm_entry::ExtractedDmmEntry;
use crate::dmm::types::parsed_pages::ParsedPages;

pub struct DmmFileEntryProcessor {
    dmm_service: Arc<dyn DmmService + Send + Sync>,
    repo_root: String,
    existing_pages: DashMap<String, i32>,
    new_pages: DashMap<String, i32>,
}

lazy_static! {
    static ref HASH_IFRAME_REGEX: Regex = {
        let pattern = r#"<iframe src="https://debridmediamanager\.com/hashlist#(.*?)"></iframe>"#;
        Regex::new(pattern).expect("Failed to compile Iframe Regex")
    };
}

impl DmmFileEntryProcessor {
    pub fn new(dmm_service: Arc<dyn DmmService + Send + Sync>, repo_root: String) -> Self {
        Self {
            dmm_service,
            repo_root,
            existing_pages: DashMap::new(),
            new_pages: DashMap::new(),
        }
    }

    pub async fn perform_scrape(&self) -> anyhow::Result<()> {
        self.load_parsed_pages().await?;
        let _entries = self.process_files().await?;
        Ok(())
    }

    async fn load_parsed_pages(&self) -> anyhow::Result<()> {
        let parsed = self.get_dmm_pages().await?;
        for page in parsed {
            self.existing_pages.insert(page.page, page.entry_count);
        }
        info!("Loaded {} previously parsed pages", self.existing_pages.len());
        Ok(())
    }

    async fn process_files(&self) -> anyhow::Result<Vec<ExtractedDmmEntry>> {
        let files_to_process = self.get_pages_from_repo_root().await?;

        let mut all_entries = Vec::new();

        for file in &files_to_process {
            let filename = Path::new(file)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or_default()
                .to_string();

            if self.existing_pages.contains_key(&filename) || self.new_pages.contains_key(&filename) {
                continue;
            }

            match self.process_page(file, &filename).await {
                Ok(entries) => {
                    all_entries.extend(entries);
                }
                Err(err) => {
                    error!("Failed to process file {}: {:?}", filename, err);
                }
            }
        }

        Ok(all_entries)
    }

    async fn process_page(&self, file: &str, filename_only: &str) -> anyhow::Result<Vec<ExtractedDmmEntry>> {
        if !Path::new(file).exists() {
            return Ok(vec![]);
        }
        
        info!("Processing file: {}", filename_only);

        let page_source = read_to_string(file).await?;
        let Some(caps) = HASH_IFRAME_REGEX.captures(&page_source) else {
            self.add_parsed_page(filename_only, 0).await?;
            return Ok(vec![]);
        };

        let hash_data = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        let decompressed_data = lz_str::decompress_from_encoded_uri_component(hash_data).expect("`compressed_data` is invalid");
        let json_str = String::from_utf16(&decompressed_data).expect("`decompressed_data` is not valid UTF16");

        let parsed = self.parse_json_entries(&json_str)?;
        let sanitized: Vec<_> = parsed
            .into_iter()
            .filter(|e| e.filesize > 0)
            .collect();
        
        info!("Found {} entries in file: {}", sanitized.len(), filename_only);

        self.add_parsed_page(filename_only, sanitized.len() as i32).await?;

        Ok(sanitized)
    }

    async fn add_parsed_page(&self, filename: &str, entry_count: i32) -> anyhow::Result<()> {
        let parsed_page = ParsedPages {
            page: filename.to_string(),
            entry_count,
        };

        self.dmm_service
            .add_page_to_ingested(&parsed_page)
            .await?;

        self.new_pages.insert(filename.to_string(), entry_count);
        Ok(())
    }

    async fn get_dmm_pages(&self) -> anyhow::Result<Vec<ParsedPages>> {
        let pages = self.dmm_service.get_ingested_pages().await?;
        Ok(pages)
    }

    async fn get_pages_from_repo_root(&self) -> anyhow::Result<Vec<String>> {
        let mut pages = Vec::new();

        if !Path::new(&self.repo_root).exists() {
            error!("DMM repo root does not exist: {}", self.repo_root);
            return Ok(pages);
        }

        let mut dir = read_dir(&self.repo_root).await?;

        while let Some(entry) = dir.next_entry().await? {
            let file_type = entry.file_type().await?;
            if file_type.is_file() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".html") && name != "index.html" {
                        pages.push(path.to_string_lossy().into_owned());
                    }
                }
            }
        }
        
        info!("Found {} DMM pages", pages.len());

        Ok(pages)
    }

    fn parse_json_entries(&self, json_str: &str) -> anyhow::Result<Vec<ExtractedDmmEntry>> {
        let json: Value = serde_json::from_str(json_str)?;

        let torrents = match &json {
            Value::Array(arr) => arr,
            Value::Object(map) => match map.get("torrents") {
                Some(Value::Array(arr)) => arr,
                _ => return Ok(vec![]),
            },
            _ => return Ok(vec![]),
        };

        let mut entries = Vec::with_capacity(torrents.len());

        for item in torrents {
            let entry = ExtractedDmmEntry::from_value(item)?;
            entries.push(entry);
        }

        Ok(entries)
    }
}