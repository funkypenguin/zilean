use serde::{Deserialize, Serialize};
use crate::dmm::types::imdb_file::ImdbFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfo {
    #[serde(rename = "raw_title")]
    pub raw_title: Option<String>,

    #[serde(rename = "parsed_title")]
    pub parsed_title: Option<String>,

    #[serde(rename = "normalized_title")]
    pub normalized_title: Option<String>,

    #[serde(rename = "cleaned_parsed_title")]
    pub cleaned_parsed_title: Option<String>,

    #[serde(default)]
    pub trash: Option<bool>,

    #[serde(default)]
    pub year: Option<i32>,

    pub resolution: Option<String>,

    #[serde(default)]
    pub seasons: Vec<i32>,

    #[serde(default)]
    pub episodes: Vec<i32>,

    #[serde(default)]
    pub complete: Option<bool>,

    #[serde(default)]
    pub volumes: Vec<i32>,

    #[serde(default)]
    pub languages: Vec<String>,

    pub quality: Option<String>,

    #[serde(default)]
    pub hdr: Vec<String>,

    pub codec: Option<String>,

    #[serde(default)]
    pub audio: Vec<String>,

    #[serde(default)]
    pub channels: Vec<String>,

    #[serde(default)]
    pub dubbed: Option<bool>,

    #[serde(default)]
    pub subbed: Option<bool>,

    pub date: Option<String>,
    pub group: Option<String>,
    pub edition: Option<String>,

    #[serde(rename = "bit_depth")]
    pub bit_depth: Option<String>,

    pub bitrate: Option<String>,
    pub network: Option<String>,

    #[serde(default)]
    pub extended: Option<bool>,

    #[serde(default)]
    pub converted: Option<bool>,

    #[serde(default)]
    pub hardcoded: Option<bool>,

    pub region: Option<String>,

    #[serde(default)]
    pub ppv: Option<bool>,

    #[serde(rename = "_3d", default)]
    pub is_3d: Option<bool>,

    pub site: Option<String>,
    pub size: Option<String>,

    #[serde(default)]
    pub proper: Option<bool>,

    #[serde(default)]
    pub repack: Option<bool>,

    #[serde(default)]
    pub retail: Option<bool>,

    #[serde(default)]
    pub upscaled: Option<bool>,

    #[serde(default)]
    pub remastered: Option<bool>,

    #[serde(default)]
    pub unrated: Option<bool>,

    #[serde(default)]
    pub documentary: Option<bool>,

    #[serde(rename = "episode_code")]
    pub episode_code: Option<String>,

    pub country: Option<String>,
    pub container: Option<String>,
    pub extension: Option<String>,

    #[serde(default)]
    pub torrent: Option<bool>,

    pub category: String,

    #[serde(rename = "imdb_id")]
    pub imdb_id: Option<String>,

    pub imdb: Option<ImdbFile>,

    #[serde(rename = "info_hash")]
    pub info_hash: String,

    #[serde(rename = "adult")]
    pub is_adult: bool,

    #[serde(rename = "ingested_at", with = "time::serde::rfc3339")]
    pub ingested_at: time::OffsetDateTime,
}