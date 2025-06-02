use crate::errors::ParserError;
use rayon::prelude::*;
use std::panic::{AssertUnwindSafe, catch_unwind};
use tracing::error;

pub mod errors;
pub mod extensions;
pub mod handler_wrapper;
pub mod parser;
pub mod parser_handlers;
pub mod transforms;
pub mod types;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParsedTitle {
    pub title: String,
    pub resolution: Option<String>,
    pub date: Option<String>,
    pub year: Option<i32>,
    pub ppv: bool,
    pub trash: bool,
    pub adult: bool,
    pub edition: Option<String>,
    pub extended: bool,
    pub convert: bool,
    pub hardcoded: bool,
    pub proper: bool,
    pub repack: bool,
    pub retail: bool,
    pub remastered: bool,
    pub unrated: bool,
    pub region: Option<String>,
    pub quality: Option<types::Quality>,
    pub bitrate: Option<String>,
    pub bit_depth: Option<String>,
    pub hdr: Vec<String>,
    pub codec: Option<types::Codec>,
    pub audio: Vec<String>,
    pub channels: Vec<String>,
    pub group: Option<String>,
    pub container: Option<String>,
    pub volumes: Vec<i32>,
    pub seasons: Vec<i32>,
    pub episodes: Vec<i32>,
    pub episode_code: Option<String>,
    pub complete: bool,
    pub languages: Vec<types::Language>,
    pub dubbed: bool,
    pub site: Option<String>,
    pub extension: Option<String>,
    pub subbed: bool,
    pub documentary: bool,
    pub upscaled: bool,
    pub is_3d: bool,
    pub extras: Vec<String>,
    pub size: Option<String>,
    pub network: Option<types::Network>,
    pub scene: bool,
}

pub fn parse_title(raw_title: &str) -> Result<ParsedTitle, ParserError> {
    let parser = parser::Parser::default();

    match catch_unwind(AssertUnwindSafe(|| parser.parse(raw_title))) {
        Ok(inner_result) => inner_result,
        Err(_) => {
            error!("Panic occurred while parsing title: '{}'", raw_title);
            Err(ParserError::Panic)
        }
    }
}

pub fn parse_batch(titles: Vec<&str>) -> Vec<Result<ParsedTitle, ParserError>> {
    titles
        .par_iter()
        .map(|title| {
            let parser = parser::Parser::default();
            parser.parse(title)
        })
        .collect()
}
