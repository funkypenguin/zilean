use async_stream::try_stream;
use dashmap::DashMap;
use futures_core::stream::Stream;
use lazy_static::lazy_static;
use parsett_rust::parse_title;
use regex::Regex;
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::{read_dir, read_to_string};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tracing::{debug, error, info};
use crate::dmm::dmm_service::DmmService;
use crate::dmm::types::parsed_pages::ParsedPages;
use crate::grpc::mapping::map_torrent_info;
use crate::imdb::searcher::ImdbSearcher;
use crate::proto::{ParsedDmmPageEntry, TorrentInfo};

pub struct DmmFileEntryProcessor {
    dmm_service: Arc<dyn DmmService + Send + Sync>,
    imdb_searcher: Arc<RwLock<ImdbSearcher>>,
    repo_root: String,
    existing_pages: DashMap<String, i32>,
    new_pages: DashMap<String, i32>,
}

lazy_static! {
    static ref HASH_IFRAME_REGEX: Regex = Regex::new(
        r#"<iframe\s+src="https://debridmediamanager\.com/hashlist#([^"]+)"[^>]*>"#
    ).expect("Failed to compile Iframe Regex");
}

impl DmmFileEntryProcessor {
    pub fn new(
        dmm_service: Arc<dyn DmmService + Send + Sync>,
        imdb_searcher: Arc<RwLock<ImdbSearcher>>,
        repo_root: String,
    ) -> Self {
        Self {
            dmm_service,
            imdb_searcher,
            repo_root,
            existing_pages: DashMap::new(),
            new_pages: DashMap::new(),
        }
    }

    pub fn stream_parsed_pages(
        self: Arc<Self>,
    ) -> impl Stream<Item=Result<ParsedDmmPageEntry, tonic::Status>> + Send + 'static {
        try_stream! {
            let filenames = match self.get_pages_from_repo_root().await {
                Ok(f) => f,
                Err(e) => {
                    error!(?e, "Failed to list repo root");
                    return;
                }
            };

            self.load_parsed_pages().await.map_err(Self::internal_error)?;

            for file in &filenames {
                let filename = Path::new(file)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or_default()
                    .to_string();

                if self.existing_pages.contains_key(&filename) || self.new_pages.contains_key(&filename) {
                    continue;
                }

                let mut count = 0;
                let stream = self.clone().process_page_stream(file.clone(), filename.clone());
                tokio::pin!(stream);

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(torrent) => {
                            yield ParsedDmmPageEntry {
                                filename: filename.clone(),
                                torrent_info: torrent.into(),
                            };
                            count += 1;
                        }
                        Err(err) => {
                            error!(?err, "Error processing entry for {filename}");
                        }
                    }
                }

                self.add_parsed_page(&filename, count).await.map_err(Self::internal_error)?;
            }
        }
    }

    pub fn process_page_stream(
        self: Arc<Self>,
        file: String,
        filename_only: String,
    ) -> impl Stream<Item=Result<TorrentInfo, anyhow::Error>> + Send + 'static {
        let imdb_searcher = Arc::clone(&self.imdb_searcher);
        try_stream! {
            if !Path::new(&file).exists() {
                return;
            }

            info!("Processing file: {}", filename_only);

            let page_source = read_to_string(&file).await?;
            let Some(caps) = HASH_IFRAME_REGEX.captures(&page_source) else {
                debug!("No hash data found in file: {}", filename_only);
                return;
            };

            let hash_data = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let decompressed_data =
                lz_str::decompress_from_encoded_uri_component(hash_data).expect("invalid hash data");
            let json_str = String::from_utf16(&decompressed_data).expect("invalid UTF16");

            let json: Value = serde_json::from_str(&json_str)?;
            let torrents = match &json {
                Value::Array(arr) => arr,
                Value::Object(map) => match map.get("torrents") {
                    Some(Value::Array(arr)) => arr,
                    _ => return,
                },
                _ => return,
            };

            for item in torrents {
                let Some(title) = item.get("filename").and_then(|t| t.as_str()) else {
                    error!("Missing or invalid filename in item: {:?}", item);
                    continue;
                };

                let Some(info_hash) = item.get("hash").and_then(|h| h.as_str()) else {
                    error!("Missing or invalid hash in item: {:?}", item);
                    continue;
                };

                let Some(bytes) = item.get("bytes").and_then(|b| b.as_i64()) else {
                    error!("Missing or invalid bytes in item: {:?}", item);
                    continue;
                };

                let parsed_entry = match parse_title(title) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Failed to parse title '{}': {:?}", title, e);
                        continue;
                    }
                };

                let mut torrent_info = map_torrent_info(info_hash, title, bytes, parsed_entry);

                let searcher = imdb_searcher.read().await;
                let imdb_match = searcher
                    .search(
                        &torrent_info.normalized_title,
                        &torrent_info.category,
                        torrent_info.year.unwrap_or_default(),
                    )
                    .into_iter()
                    .next();

                if let Some(best) = imdb_match {
                    torrent_info.imdb_id = Some(best.imdb_id);
                }

                yield torrent_info;
            }
        }
    }

    async fn load_parsed_pages(&self) -> anyhow::Result<()> {
        let parsed = self.dmm_service.get_ingested_pages().await?;
        for page in parsed {
            self.existing_pages.insert(page.page, page.entry_count);
        }
        Ok(())
    }

    async fn add_parsed_page(&self, filename: &str, entry_count: i32) -> anyhow::Result<()> {
        self.dmm_service
            .add_page_to_ingested(&ParsedPages {
                page: filename.to_string(),
                entry_count,
            })
            .await?;
        self.new_pages.insert(filename.to_string(), entry_count);
        Ok(())
    }

    async fn get_pages_from_repo_root(&self) -> anyhow::Result<Vec<String>> {
        let mut pages = Vec::new();
        if !Path::new(&self.repo_root).exists() {
            return Ok(pages);
        }
        let mut dir = read_dir(&self.repo_root).await?;
        while let Some(entry) = dir.next_entry().await? {
            if entry.file_type().await?.is_file() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".html") && name != "index.html" {
                        pages.push(path.to_string_lossy().into_owned());
                    }
                }
            }
        }

        debug!("Found {} DMM HTML files in repo", pages.len());

        Ok(pages)
    }


    fn internal_error(err: anyhow::Error) -> tonic::Status {
        tonic::Status::internal(err.to_string())
    }
}
