use crate::{proto, utils};
use crate::proto::{TorrentInfo, TorrentTitleResponse};
use parsett_rust::ParsedTitle;

pub fn map_parsed_title(
    info_hash: &str,
    original_title: &str,
    parsed: ParsedTitle,
) -> TorrentTitleResponse {
    TorrentTitleResponse {
        info_hash: info_hash.into(),
        original_title: original_title.into(),
        title: parsed.title,
        resolution: parsed.resolution,
        date: parsed.date,
        year: parsed.year,
        ppv: parsed.ppv,
        trash: parsed.trash,
        adult: parsed.adult,
        edition: parsed.edition,
        extended: parsed.extended,
        convert: parsed.convert,
        hardcoded: parsed.hardcoded,
        proper: parsed.proper,
        repack: parsed.repack,
        retail: parsed.retail,
        remastered: parsed.remastered,
        unrated: parsed.unrated,
        region: parsed.region,
        quality: parsed.quality.map(|q| proto::Quality::from(q) as i32),
        bitrate: parsed.bitrate,
        bit_depth: parsed.bit_depth,
        hdr: parsed.hdr,
        codec: parsed.codec.map(|c| proto::Codec::from(c) as i32),
        audio: parsed.audio,
        channels: parsed.channels,
        group: parsed.group,
        container: parsed.container,
        volumes: parsed.volumes,
        seasons: parsed.seasons,
        episodes: parsed.episodes,
        episode_code: parsed.episode_code,
        complete: parsed.complete,
        languages: parsed
            .languages
            .into_iter()
            .map(|l| proto::Language::from(l) as i32)
            .collect(),
        dubbed: parsed.dubbed,
        site: parsed.site,
        extension: parsed.extension,
        subbed: parsed.subbed,
        documentary: parsed.documentary,
        upscaled: parsed.upscaled,
        is_3d: parsed.is_3d,
        extras: parsed.extras,
        size: parsed.size,
        network: parsed.network.map(|n| proto::Network::from(n) as i32),
        scene: parsed.scene,
    }
}

pub fn map_torrent_info(
    info_hash: &str,
    original_title: &str,
    bytes: i64,
    parsed: ParsedTitle,
) -> TorrentInfo {
    let ParsedTitle {
        title,
        resolution,
        date,
        year,
        ppv,
        trash,
        adult,
        edition,
        extended,
        convert,
        hardcoded,
        proper,
        repack,
        retail,
        remastered,
        unrated,
        region,
        bitrate,
        bit_depth,
        hdr,
        audio,
        channels,
        group,
        container,
        volumes,
        seasons,
        episodes,
        episode_code,
        complete,
        dubbed,
        site,
        extension,
        subbed,
        documentary,
        upscaled,
        is_3d,
        extras,
        size: _,
        scene,
        network,
        codec,
        quality,
        languages,
    } = parsed;

    let category = assign_category(adult, &seasons, &episodes);
    let parsed_title = title;
    let normalized_title = utils::strings::normalize_title(&parsed_title);

    TorrentInfo {
        raw_title: original_title.into(),
        parsed_title,
        normalized_title,
        cleaned_parsed_title: None,
        info_hash: info_hash.into(),
        resolution,
        date,
        year,
        ppv,
        trash,
        is_adult: adult,
        edition,
        extended,
        convert,
        hardcoded,
        proper,
        repack,
        retail,
        remastered,
        unrated,
        region,
        bitrate,
        bit_depth,
        hdr,
        audio,
        channels,
        group,
        container,
        volumes,
        seasons,
        episodes,
        episode_code,
        complete,
        dubbed,
        site,
        extension,
        torrent: None,
        category,
        subbed,
        documentary,
        upscaled,
        is_3d,
        extras,
        size: Some(bytes.to_string()),
        scene,
        country: None,
        imdb_id: None,
        ingested_at: chrono::Utc::now().to_string(),
        network: network.map(|n| proto::Network::from(n) as i32),
        codec: codec.map(|c| proto::Codec::from(c) as i32),
        quality: quality.map(|q| proto::Quality::from(q) as i32),
        languages: languages
            .into_iter()
            .map(|l| proto::Language::from(l) as i32)
            .collect(),
    }
}

fn assign_category(adult: bool, seasons: &[i32], episodes: &[i32]) -> String {
    if adult {
        "xxx".to_string()
    } else if seasons.is_empty() && episodes.is_empty() {
        "movie".to_string()
    } else {
        "tvSeries".to_string()
    }
}

pub fn map_to_empty_on_error(info_hash: &str, original_title: &str) -> TorrentTitleResponse {
    TorrentTitleResponse {
        info_hash: info_hash.into(),
        original_title: original_title.into(),
        ..Default::default()
    }
}

impl From<parsett_rust::types::Codec> for proto::Codec {
    fn from(value: parsett_rust::types::Codec) -> Self {
        proto::Codec::try_from(value as i32).unwrap_or(proto::Codec::Unknown)
    }
}

impl From<parsett_rust::types::Network> for proto::Network {
    fn from(value: parsett_rust::types::Network) -> Self {
        proto::Network::try_from(value as i32).unwrap_or(proto::Network::Unknown)
    }
}

impl From<parsett_rust::types::Language> for proto::Language {
    fn from(value: parsett_rust::types::Language) -> Self {
        proto::Language::try_from(value as i32).unwrap_or(proto::Language::LangUnknown)
    }
}

impl From<parsett_rust::types::Quality> for proto::Quality {
    fn from(value: parsett_rust::types::Quality) -> Self {
        proto::Quality::try_from(value as i32).unwrap_or(proto::Quality::Unknown)
    }
}
