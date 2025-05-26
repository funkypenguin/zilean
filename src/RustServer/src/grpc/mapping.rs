use crate::proto;
use crate::proto::TorrentTitleResponse;
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
