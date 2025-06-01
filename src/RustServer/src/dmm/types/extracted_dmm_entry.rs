#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::dmm::types::streamed_entry::StreamedEntry;
use crate::dmm::types::torrent_info::TorrentInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedDmmEntry {
    #[serde(rename = "hash")]
    pub info_hash: Option<String>,

    #[serde(rename = "filename")]
    pub filename: Option<String>,

    #[serde(rename = "bytes")]
    pub filesize: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_response: Option<TorrentInfo>,
}

impl ExtractedDmmEntry {
    pub fn from_streamed_entry(entry: StreamedEntry) -> Self {
        Self {
            info_hash: Some(entry.info_hash),
            filename: Some(entry.name),
            filesize: entry.size,
            parse_response: None,
        }
    }

    pub fn from_value(value: &Value) -> anyhow::Result<Self> {
        let info_hash = value.get("hash").and_then(|v| v.as_str()).map(String::from);
        let filename = value.get("filename").and_then(|v| v.as_str()).map(|f| f.replace('.', " "));
        let filesize = value.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);

        Ok(Self {
            info_hash,
            filename,
            filesize,
            parse_response: None,
        })
    }
}