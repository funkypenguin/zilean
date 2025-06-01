use serde::{Deserialize, Serialize};
use crate::dmm::types::torrent_info::TorrentInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedDmmEntryResponse {
    pub filename: Option<String>,
    pub info_hash: Option<String>,
    pub filesize: String,
    pub parse_response: TorrentInfo,
}