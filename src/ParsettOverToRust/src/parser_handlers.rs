use regress::Regex;
use std::cmp::min;

use lazy_static::lazy_static;
use crate::extensions::regex::RegexStringExt;
use crate::handler_wrapper::{Handler, HandlerResult, Match, RegexHandlerOptions};
use crate::{parser, transforms};
use crate::types::{Codec, Language, Network, Quality};

fn load_embedded_adult_keywords() -> Vec<String> {
    include_str!("../adult_word_lists/combined.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(regex::escape)
        .collect()
}

lazy_static! {
    static ref ADULT_KEYWORDS_PATTERN: Regex = {
        let escaped = load_embedded_adult_keywords();
        let pattern = format!(r"\b(?:{})\b", escaped.join("|"));
        Regex::new(&pattern).expect("Failed to compile embedded adult keyword regex")
    };
}

lazy_static! {
        static ref VOLUME_REGEX: Regex =
            Regex::case_insensitive(r"\bvol(?:ume)?[. -]*(\d{1,2})").unwrap();
    }

lazy_static! {
        static ref EPISODE_RE1: Regex = Regex::case_insensitive(
            r"(?<!movie\W*|film\W*|^)(?:[ .]+-[ .]+|[([][ .]*)(\d{1,4})(?:a|b|v\d|\.\d)?(?:\W|$)(?!movie|film|\d+)"
        )
        .unwrap();
        static ref EPISODE_RE2: Regex = Regex::case_insensitive(r"^(?:[\[\(-][ .]?)?(\d{1,4})(?:a|b|v\d)?(?:\W|$)(?!movie|film)").unwrap();
        static ref EPISODE_RE3: Regex = Regex::new(r"\d+").unwrap();
    }

lazy_static! {
        static ref PT_LANG_RE1: Regex = Regex::case_insensitive(r"capitulo|ao").unwrap();
        static ref PT_LANG_RE2: Regex = Regex::case_insensitive(r"dublado").unwrap();
    }

pub fn add_default_handlers(parser: &mut parser::Parser) {
    // Adult
    parser.add_handler(Handler::from_regex(
        "adult",
        |t| &mut t.adult,
        Regex::case_insensitive(r"\b(?:xxx|xx)\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            skip_from_title: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_static_regex(
        "adult",
        |t| &mut t.adult,
        &ADULT_KEYWORDS_PATTERN,
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            skip_from_title: true,
            skip_if_already_found: true,
            ..Default::default()
        },
    ));

    // Scene
    parser.add_handler(Handler::from_regex(
        "scene",
        |t| &mut t.scene,
        Regex::new(r"^(?=.*(\b\d{3,4}p\b).*([_. ]WEB[_. ])(?!DL)\b)|\b(-CAKES|-GGEZ|-GGWP|-GLHF|-GOSSIP|-NAISU|-KOGI|-PECULATE|-SLOT|-EDITH|-ETHEL|-ELEANOR|-B2B|-SPAMnEGGS|-FTP|-DiRT|-SYNCOPY|-BAE|-SuccessfulCrab|-NHTFS|-SURCODE|-B0MBARDIERS)").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));

    // Extras (this stuff can be trashed)
    parser.add_handler(Handler::from_regex(
        "extras",
        |t| &mut t.extras,
        Regex::case_insensitive(r"\bNCED\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("NCED"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "extras",
        |t| &mut t.extras,
        Regex::case_insensitive(r"\bNCOP\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("NCOP"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "extras",
        |t| &mut t.extras,
        Regex::case_insensitive(r"\b(?:Deleted[ .-]*)?Scene(?:s)?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_value("Deleted Scene"),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "extras",
        |t| &mut t.extras,
        Regex::case_insensitive(r"(?:(?<=\b(?:19\d{2}|20\d{2})\b.*)\b(?:Featurettes?)\b|\bFeaturettes?\b(?!.*\b(?:19\d{2}|20\d{2})\b))")
            .unwrap(),
        transforms::chain_transforms(transforms::replace_value("Featurette"), transforms::uniq_concat),
        RegexHandlerOptions {
            skip_from_title: true,
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "extras",
        |t| &mut t.extras,
        Regex::case_insensitive(r"(?:(?<=\b(?:19\d{2}|20\d{2})\b.*)\b(?:Sample)\b|\b(?:Sample)\b(?!.*\b(?:19\d{2}|20\d{2})\b))").unwrap(),
        transforms::chain_transforms(transforms::replace_value("Sample"), transforms::uniq_concat),
        RegexHandlerOptions {
            skip_from_title: true,
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "extras",
        |t| &mut t.extras,
        Regex::case_insensitive(
            r"(?:(?<=\b(?:19\d{2}|20\d{2})\b.*)\b(?:Trailers?)\b|\bTrailers?\b(?!.*\b(?:19\d{2}|20\d{2}|.(Park|And))\b))",
        )
            .unwrap(),
        transforms::chain_transforms(transforms::replace_value("Trailer"), transforms::uniq_concat),
        RegexHandlerOptions {
            skip_from_title: true,
            remove: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "ppv",
        |t| &mut t.ppv,
        Regex::case_insensitive(r"\bPPV\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            skip_from_title: true,
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "ppv",
        |t| &mut t.ppv,
        Regex::case_insensitive(r"\b\W?Fight.?Nights?\W?\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            skip_from_title: true,
            remove: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "site",
        |t| &mut t.site,
        Regex::case_insensitive(r"^(www?[\.,][\w-]+\.[\w-]+(?:\.[\w-]+)?)\s+-\s*").unwrap(),
        transforms::identity,
        RegexHandlerOptions {
            skip_from_title: true,
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "site",
        |t| &mut t.site,
        Regex::case_insensitive(r"^((?:www?[\.,])?[\w-]+\.[\w-]+(?:\.[\w-]+)*?)\s+-\s*").unwrap(),
        transforms::identity,
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "episode_code",
        |t| &mut t.episode_code,
        Regex::new(r"[[(]([a-zA-Z0-9]{8})[\])](?=\.[a-zA-Z0-9]{1,5}$|$)").unwrap(),
        transforms::uppercase,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "episode_code",
        |t| &mut t.episode_code,
        Regex::new(r"\[([A-Z0-9]{8})]").unwrap(),
        transforms::uppercase,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"\[?\]?3840x\d{4}[\])?]?").unwrap(),
        transforms::value("2160p"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"\[?\]?1920x\d{3,4}[\])?]?").unwrap(),
        transforms::value("1080p"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"\[?\]?1280x\d{3}[\])?]?").unwrap(),
        transforms::value("720p"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"\[?\]?(\d{3,4}x\d{3,4})[\])?]?p?").unwrap(),
        transforms::value("$1p"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"(480|720|1080)0[pi]").unwrap(),
        transforms::value("$1p"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"(?:QHD|QuadHD|WQHD|2560(\d+)?x(\d+)?1440p?)").unwrap(),
        transforms::value("1440p"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"(?:Full HD|FHD|1920(\d+)?x(\d+)?1080p?)").unwrap(),
        transforms::value("1080p"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"(?:BD|HD|M)(2160p?|4k)").unwrap(),
        transforms::value("2160p"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"(?:BD|HD|M)1080p?").unwrap(),
        transforms::value("1080p"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"(?:BD|HD|M)720p?").unwrap(),
        transforms::value("720p"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"(?:BD|HD|M)480p?").unwrap(),
        transforms::value("480p"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(
            r"\b(?:4k|2160p|1080p|720p|480p)(?!.*\b(?:4k|2160p|1080p|720p|480p)\b)",
        )
            .unwrap(),
        transforms::resolution_transform,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"\b4k|21600?[pi]\b").unwrap(),
        transforms::value("2160p"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"(\d{3,4})[pi]").unwrap(),
        transforms::value("$1p"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "resolution",
        |t| &mut t.resolution,
        Regex::case_insensitive(r"(240|360|480|576|720|1080|2160|3840)[pi]").unwrap(),
        transforms::lowercase,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "trash",
        |t| &mut t.trash,
        Regex::case_insensitive(
            r"\b(?:H[DQ][ .-]*)?CAM(?!.?(S|E|\()\d+)(?:H[DQ])?(?:[ .-]*Rip|Rp)?\b",
        )
            .unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "trash",
        |t| &mut t.trash,
        Regex::case_insensitive(r"\b(?:H[DQ][ .-]*)?S[ \.\-]print\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "trash",
        |t| &mut t.trash,
        Regex::case_insensitive(r"\b(?:HD[ .-]*)?T(?:ELE)?(C|S)(?:INE|YNC)?(?:Rip)?\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "trash",
        |t| &mut t.trash,
        Regex::case_insensitive(r"\bPre.?DVD(?:Rip)?\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "trash",
        |t| &mut t.trash,
        Regex::case_insensitive(r"\b(?:DVD?|BD|BR)?[ .-]*Scr(?:eener)?\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "trash",
        |t| &mut t.trash,
        Regex::case_insensitive(r"\bDVB[ .-]*(?:Rip)?\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "trash",
        |t| &mut t.trash,
        Regex::case_insensitive(r"\bSAT[ .-]*Rips?\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "trash",
        |t| &mut t.trash,
        Regex::case_insensitive(r"\bLeaked\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "trash",
        |t| &mut t.trash,
        Regex::case_insensitive(r"threesixtyp").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "trash",
        |t| &mut t.trash,
        Regex::case_insensitive(r"\bR5|R6\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "trash",
        |t| &mut t.trash,
        Regex::case_insensitive(r"\b(?:Deleted[ .-]*)?Scene(?:s)?\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "trash",
        |t| &mut t.trash,
        Regex::case_insensitive(r"\bHQ.?(Clean)?.?(Aud(io)?)?\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "trash",
        |t| &mut t.trash,
        Regex::case_insensitive(r"от\s*New-?Team").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    // Date
    parser.add_handler(Handler::from_regex(
        "date",
        |t| &mut t.date,
        Regex::new(r"(?:\W|^)([[(]?(?:19[6-9]|20[012])[0-9]([. \-/\\])(?:0[1-9]|1[012])\2(?:0[1-9]|[12][0-9]|3[01])[\]\)]?)(?:\W|$)")
            .unwrap(),
        transforms::date_from_format("%Y %m %d"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "date",
        |t| &mut t.date,
        Regex::new(r"(?:\W|^)(\[?\]?(?:0[1-9]|[12][0-9]|3[01])([. \-/\\])(?:0[1-9]|1[012])\2(?:19[6-9]|20[01])[0-9][\])]?)(?:\W|$)")
            .unwrap(),
        transforms::date_from_format("%d %m %Y"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "date",
        |t| &mut t.date,
        Regex::new(r"(?:\W)(\[?\]?(?:0[1-9]|1[012])([. \-/\\])(?:0[1-9]|[12][0-9]|3[01])\2(?:[0][1-9]|[0126789][0-9])[\])]?)(?:\W|$)")
            .unwrap(),
        transforms::date_from_format("%m %d %y"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "date",
        |t| &mut t.date,
        Regex::new(r"(?:\W)(\[?\]?(?:0[1-9]|[12][0-9]|3[01])([. \-/\\])(?:0[1-9]|1[012])\2(?:[0][1-9]|[0126789][0-9])[\])]?)(?:\W|$)")
            .unwrap(),
        transforms::date_from_format("%d %m %y"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "date",
        |t| &mut t.date,
        Regex::case_insensitive(r"(?:\W|^)([([]?(?:0?[1-9]|[12][0-9]|3[01])[. ]?(?:st|nd|rd|th)?([. \-/\\])(?:feb(?:ruary)?|jan(?:uary)?|mar(?:ch)?|apr(?:il)?|may|june?|july?|aug(?:ust)?|sept?(?:ember)?|oct(?:ober)?|nov(?:ember)?|dec(?:ember)?)\2(?:19[7-9]|20[012])[0-9][)\]]?)(?=\W|$)").unwrap(),
        transforms::date_from_formats(&[
            "%d %b %Y",      // regular format (9 Dec 2019)
            "%dst %b %Y",    // for 1st, 21st, 31st
            "%dnd %b %Y",    // for 2nd, 22nd
            "%drd %b %Y",    // for 3rd, 23rd
            "%dth %b %Y",    // for 4th, 5th, etc.
            "%d %B %Y",      // full month name without ordinal
            "%dst %B %Y",    // full month name versions
            "%dnd %B %Y",
            "%drd %B %Y",
            "%dth %B %Y",
        ]),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "date",
        |t| &mut t.date,
        Regex::case_insensitive(r"(?:\W|^)(\[?\]?(?:0?[1-9]|[12][0-9]|3[01])[. ]?(?:st|nd|rd|th)?([. \-\/\\])(?:feb(?:ruary)?|jan(?:uary)?|mar(?:ch)?|apr(?:il)?|may|june?|july?|aug(?:ust)?|sept?(?:ember)?|oct(?:ober)?|nov(?:ember)?|dec(?:ember)?)\2(?:0[1-9]|[0126789][0-9])[\])]?)(?:\W|$)").unwrap(),
        transforms::date_from_format("%d %b %y"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "date",
        |t| &mut t.date,
        Regex::new(r"(?:\W|^)(\[?\]?20[012][0-9](?:0[1-9]|1[012])(?:0[1-9]|[12][0-9]|3[01])[\])]?)(?:\W|$)").unwrap(),
        transforms::date_from_format("%Y%m%d"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    // Complete
    parser.add_handler(Handler::from_regex(
        "complete",
        |t| &mut t.complete,
        Regex::new(r"\b((?:19\d|20[012])\d[ .]?-[ .]?(?:19\d|20[012])\d)\b").unwrap(), // year range
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "complete",
        |t| &mut t.complete,
        Regex::new(r"[([][ .]?((?:19\d|20[012])\d[ .]?-[ .]?\d{2})[ .]?[)\]]").unwrap(), // year range
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "bitrate",
        |t| &mut t.bitrate,
        Regex::case_insensitive(r"\b\d+[kmg]bps\b").unwrap(),
        transforms::lowercase,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "year",
        |t| &mut t.year,
        Regex::new(r"\b(20[0-9]{2}|2100)(?!\D*\d{4}\b)").unwrap(),
        transforms::parse::<i32>,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "year",
        |t| &mut t.year,
        Regex::case_insensitive(r"[([]?(?!^)(?<!\d|Cap[. ]?)((?:19\d|20[012])\d)(?!\d|kbps)[)\]]?")
            .unwrap(),
        transforms::parse::<i32>,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "year",
        |t| &mut t.year,
        Regex::case_insensitive(r"^[([]?((?:19\d|20[012])\d)(?!\d|kbps)[)\]]?").unwrap(),
        transforms::parse::<i32>,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "edition",
        |t| &mut t.edition,
        Regex::case_insensitive(
            r"\b\d{2,3}(th)?[\.\s\-\+_\/(),]Anniversary[\.\s\-\+_\/(),](Edition|Ed)?\b",
        )
            .unwrap(),
        transforms::value("Anniversary Edition"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "edition",
        |t| &mut t.edition,
        Regex::case_insensitive(r"\bUltimate[\.\s\-\+_\/(),]Edition\b").unwrap(),
        transforms::value("Ultimate Edition"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "edition",
        |t| &mut t.edition,
        Regex::case_insensitive(r#"\bExtended[\.\s\-\+_\/(),]Director\"?s\b"#).unwrap(),
        transforms::value("Directors Cut"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "edition",
        |t| &mut t.edition,
        Regex::case_insensitive(r"\b(custom.?)?Extended\b").unwrap(),
        transforms::value("Extended Edition"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "edition",
        |t| &mut t.edition,
        Regex::case_insensitive(r#"\bDirector\"?s[\.\s\-\+_\/(),]Cut\b"#).unwrap(),
        transforms::value("Directors Cut"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "edition",
        |t| &mut t.edition,
        Regex::case_insensitive(r#"\bCollector\"?s\b"#).unwrap(),
        transforms::value("Collectors Edition"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "edition",
        |t| &mut t.edition,
        Regex::case_insensitive(r"\bTheatrical\b").unwrap(),
        transforms::value("Theatrical"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "edition",
        |t| &mut t.edition,
        Regex::case_insensitive(r"\bUncut\b").unwrap(),
        transforms::value("Uncut"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "edition",
        |t| &mut t.edition,
        Regex::case_insensitive(r"\bIMAX\b").unwrap(),
        transforms::value("IMAX"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "edition",
        |t| &mut t.edition,
        Regex::case_insensitive(r"\b\.Diamond\.\b").unwrap(),
        transforms::value("Diamond Edition"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "edition",
        |t| &mut t.edition,
        Regex::case_insensitive(r"\bRemaster(?:ed)?\b").unwrap(),
        transforms::value("Remastered"),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "upscaled",
        |t| &mut t.upscaled,
        Regex::case_insensitive(r"\b(?:AI.?)?(Upscal(ed?|ing)|Enhanced?)\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "upscaled",
        |t| &mut t.upscaled,
        Regex::case_insensitive(r"\b(?:iris2|regrade|ups(uhd|fhd|hd|4k))\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "upscaled",
        |t| &mut t.upscaled,
        Regex::case_insensitive(r"\b\.AI\.\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "convert",
        |t| &mut t.convert,
        Regex::case_insensitive(r"\bCONVERT\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "hardcoded",
        |t| &mut t.hardcoded,
        Regex::case_insensitive(r"\bHC|HARDCODED\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "proper",
        |t| &mut t.proper,
        Regex::case_insensitive(r"\b(?:REAL.)?PROPER\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "repack",
        |t| &mut t.repack,
        Regex::case_insensitive(r"\bREPACK|RERIP\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "retail",
        |t| &mut t.retail,
        Regex::case_insensitive(r"\bRetail\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "remastered",
        |t| &mut t.remastered,
        Regex::case_insensitive(r"\bRemaster(?:ed)?\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "documentary",
        |t| &mut t.documentary,
        Regex::case_insensitive(r"\bDOCU(?:menta?ry)?\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            skip_from_title: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "unrated",
        |t| &mut t.unrated,
        Regex::case_insensitive(r"\bunrated|uncensored\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "region",
        |t| &mut t.region,
        Regex::new(r"R\d\b").unwrap(),
        transforms::identity,
        RegexHandlerOptions {
            skip_if_first: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\b(?:HD[ .-]*)?T(?:ELE)?S(?:YNC)?(?:Rip)?\b").unwrap(),
        transforms::const_value(Quality::TeleSync),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::new(r"\b(?:HD[ .-]*)?T(?:ELE)?C(?:INE)?(?:Rip)?\b").unwrap(),
        transforms::const_value(Quality::TeleCine),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\b(?:DVD?|BD|BR)?[ .-]*Scr(?:eener)?\b").unwrap(),
        transforms::const_value(Quality::SCR),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bP(?:RE)?-?(HD|DVD)(?:Rip)?\b").unwrap(),
        transforms::const_value(Quality::SCR),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bBlu[ .-]*Ray\b(?=.*remux)").unwrap(),
        transforms::const_value(Quality::BluRayRemux),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"(?:BD|BR|UHD)[- ]?remux").unwrap(),
        transforms::const_value(Quality::BluRayRemux),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"(?<=remux.*)\bBlu[ .-]*Ray\b").unwrap(),
        transforms::const_value(Quality::BluRayRemux),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bremux\b").unwrap(),
        transforms::const_value(Quality::Remux),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bBlu[ .-]*Ray\b(?![ .-]*Rip)").unwrap(),
        transforms::const_value(Quality::BluRay),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bUHD[ .-]*Rip\b").unwrap(),
        transforms::const_value(Quality::UHDRip),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bHD[ .-]*Rip\b").unwrap(),
        transforms::const_value(Quality::HDRip),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bMicro[ .-]*HD\b").unwrap(),
        transforms::const_value(Quality::HDRip),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\b(?:BR|Blu[ .-]*Ray)[ .-]*Rip\b").unwrap(),
        transforms::const_value(Quality::BRRip),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bBD[ .-]*Rip\b|\bBDR\b|\bBD-RM\b|[[(]BD[\]) .,-]").unwrap(),
        transforms::const_value(Quality::BDRip),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\b(?:HD[ .-]*)?DVD[ .-]*Rip\b").unwrap(),
        transforms::const_value(Quality::DVDRip),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bVHS[ .-]*Rip?\b").unwrap(),
        transforms::const_value(Quality::VHSRip),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bDVD(?:R\d?|.*Mux)?\b").unwrap(),
        transforms::const_value(Quality::DVD),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bVHS\b").unwrap(),
        transforms::const_value(Quality::VHS),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bPPVRip\b").unwrap(),
        transforms::const_value(Quality::PPVRip),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bHD.?TV.?Rip\b").unwrap(),
        transforms::const_value(Quality::HDTVRip),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bHD.?TV\b").unwrap(),
        transforms::const_value(Quality::HDTV),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bDVB[ .-]*(?:Rip)?\b").unwrap(),
        transforms::const_value(Quality::HDTV),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bSAT[ .-]*Rips?\b").unwrap(),
        transforms::const_value(Quality::SATRip),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bTVRips?\b").unwrap(),
        transforms::const_value(Quality::TVRip),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bR5\b").unwrap(),
        transforms::const_value(Quality::R5),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\b(?:DL|WEB|BD|BR)MUX\b").unwrap(),
        transforms::const_value(Quality::WebMux),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bWEB[ .-]*Rip\b").unwrap(),
        transforms::const_value(Quality::WebRip),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bWEB[ .-]?DL[ .-]?Rip\b").unwrap(),
        transforms::const_value(Quality::WebDLRip),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bWEB[ .-]*(DL|.BDrip|.DLRIP)\b").unwrap(),
        transforms::const_value(Quality::WebDL),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\b(?<!\w.)WEB\b|\bWEB(?!([ \.\-\(\],]+\d))\b").unwrap(),
        transforms::const_value(Quality::Web),
        RegexHandlerOptions {
            remove: true,
            skip_from_title: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(
            r"\b(?:H[DQ][ .-]*)?CAM(?!.?(S|E|\()\d+)(?:H[DQ])?(?:[ .-]*Rip|Rp)?\b",
        )
            .unwrap(),
        transforms::const_value(Quality::Cam),
        RegexHandlerOptions {
            remove: true,
            skip_from_title: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\b(?:H[DQ][ .-]*)?S[ \.\-]print").unwrap(),
        transforms::const_value(Quality::Cam),
        RegexHandlerOptions {
            remove: true,
            skip_from_title: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "quality",
        |t| &mut t.quality,
        Regex::case_insensitive(r"\bPDTV\b").unwrap(),
        transforms::const_value(Quality::PDTV),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "bit_depth",
        |t| &mut t.bit_depth,
        Regex::case_insensitive(r"\bhevc\s?10\b").unwrap(),
        transforms::value("10bit"),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "bit_depth",
        |t| &mut t.bit_depth,
        Regex::case_insensitive(r"(?:8|10|12)[-\.]?(?=bit)").unwrap(),
        transforms::value("$1bit"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "bit_depth",
        |t| &mut t.bit_depth,
        Regex::case_insensitive(r"\bhdr10\b").unwrap(),
        transforms::value("10bit"),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "bit_depth",
        |t| &mut t.bit_depth,
        Regex::case_insensitive(r"\bhi10\b").unwrap(),
        transforms::value("10bit"),
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::new("bit_depth", |context| {
        if let Some(bit_depth) = context.result.bit_depth.clone() {
            context.result.bit_depth = Some(bit_depth.replace("-", "").replace(" ", ""));
        }
        None
    }));

    parser.add_handler(Handler::from_regex(
        "hdr",
        |t| &mut t.hdr,
        Regex::case_insensitive(r"\bDV\b|dolby.?vision|\bDoVi\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("DV"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "hdr",
        |t| &mut t.hdr,
        Regex::case_insensitive(r"HDR10(?:\+|[-\.\s]?plus)").unwrap(),
        transforms::chain_transforms(transforms::replace_value("HDR10+"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "hdr",
        |t| &mut t.hdr,
        Regex::case_insensitive(r"\bHDR(?:10)?\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("HDR"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "hdr",
        |t| &mut t.hdr,
        Regex::case_insensitive(r"\bSDR\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("SDR"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "codec",
        |t| &mut t.codec,
        Regex::case_insensitive(r"\b[hx][\. \-]?264\b").unwrap(),
        transforms::const_value(Codec::Avc),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "codec",
        |t| &mut t.codec,
        Regex::case_insensitive(r"\b[hx][\. \-]?265\b").unwrap(),
        transforms::const_value(Codec::Hevc),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "codec",
        |t| &mut t.codec,
        Regex::case_insensitive(r"HEVC10(bit)?\b|\b[xh][\. \-]?265\b").unwrap(),
        transforms::const_value(Codec::Hevc),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "codec",
        |t| &mut t.codec,
        Regex::case_insensitive(r"\bhevc(?:\s?10)?\b").unwrap(),
        transforms::const_value(Codec::Hevc),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "codec",
        |t| &mut t.codec,
        Regex::case_insensitive(r"\bdivx|xvid\b").unwrap(),
        transforms::const_value(Codec::Xvid),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "codec",
        |t| &mut t.codec,
        Regex::case_insensitive(r"\bavc\b").unwrap(),
        transforms::const_value(Codec::Avc),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "codec",
        |t| &mut t.codec,
        Regex::case_insensitive(r"\bav1\b").unwrap(),
        transforms::const_value(Codec::Av1),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "codec",
        |t| &mut t.codec,
        Regex::case_insensitive(r"\b(?:mpe?g\d*)\b").unwrap(),
        transforms::const_value(Codec::Mpeg),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "channels",
        |t| &mut t.channels,
        Regex::case_insensitive(r"\bDDP?5[ \.\_]1\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("5.1"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "channels",
        |t| &mut t.channels,
        Regex::case_insensitive(r"\b5\.1(ch)?\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("5.1"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "channels",
        |t| &mut t.channels,
        Regex::case_insensitive(r"\b7[\.\- ]1(.?ch(annel)?)?\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("7.1"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "channels",
        |t| &mut t.channels,
        Regex::case_insensitive(r"\b2\.0\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("2.0"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "channels",
        |t| &mut t.channels,
        Regex::case_insensitive(r"\bstereo\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("stereo"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "channels",
        |t| &mut t.channels,
        Regex::case_insensitive(r"\bmono\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("mono"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "channels",
        |t| &mut t.channels,
        Regex::case_insensitive(r"\b(?:x[2-4]|5[\W]1(?:x[2-4])?)\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("5.1"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "channels",
        |t| &mut t.channels,
        Regex::case_insensitive(r"\b2\.0(?:x[2-4])\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("2.0"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\bDDP5[ \.\_]1\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_value("Dolby Digital Plus"),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_first: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\b(?!.+HR)(DTS.?HD.?Ma(ster)?|DTS.?X)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_value("DTS Lossless"),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\bDTS(?!(.?HD.?Ma(ster)?|.X)).?(HD.?HR|HD)?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_value("DTS Lossy"),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\b(Dolby.?)?Atmos\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("Atmos"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\b(TrueHD|\.True\.)\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("TrueHD"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            skip_from_title: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::new(r"\bTRUE\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("TrueHD"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            skip_from_title: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\bFLAC(?:\+?2\.0)?(x[2-4])?\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("FLAC"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\bEAC-?3(?:[. -]?[256]\.[01])?\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("EAC3"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\bAC-?3(x2)?(?:[ .-](5\.1)?[x+]2\.?0?x?3?)?\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("AC3"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\b5\.1(ch)?\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("AC3"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\b(DD2?[\+p]2?(.?5.1)?|DD Plus|Dolby Digital Plus)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_value("Dolby Digital Plus"),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\b(DD|Dolby.?Digital.?)2?(5.?1)?(?!.?(Plus|P|\+))\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_value("Dolby Digital"),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\bDolbyD\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_value("Dolby Digital"),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_first: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\bQ?Q?AAC(x?2)?\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("AAC"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |t| &mut t.audio,
        Regex::case_insensitive(r"\b(H[DQ])?.?(Clean.?Aud(io)?)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_value("HQ Clean Audio"),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "group",
        |t| &mut t.group,
        Regex::case_insensitive(r"- ?(?!\d+$|S\d+|\d+x|ep?\d+|[^[]+]$)([^\-. []+[^\-. [)\]\d][^\-. [)\]]*)(?:\[[\w.-]+])?(?=\.\w{2,4}$|$)")
            .unwrap(),
        transforms::identity,
        RegexHandlerOptions {
            remove: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "container",
        |t| &mut t.container,
        Regex::case_insensitive(r"\.?[\[(]?\b(MKV|AVI|MP4|WMV|MPG|MPEG)\b[\])]?").unwrap(),
        transforms::lowercase,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "volumes",
        |t| &mut t.volumes,
        Regex::case_insensitive(r"\bvol(?:s|umes?)?[. -]*(?:\d{1,2}[., +/\\&-]+)+\d{1,2}\b")
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::new("volumes", |context| {
        let title = &context.title;
        let start_index = context
            .matched
            .get("year")
            .map(|y| y.match_index)
            .unwrap_or(0);
        let start_index = min(start_index, title.len() - 1);

        if let Some(m) = VOLUME_REGEX.find_str(&title[start_index..]) {
            let vol = m.group(1).unwrap().as_str().parse::<i32>().unwrap();

            context.matched.insert("volumes".to_string(), Match {
                raw_match: m.as_str().to_string(),
                match_index: m.start(),
            });

            context.result.volumes = vec![vol];

            return Some(HandlerResult {
                raw_match: m.as_str().to_string(),
                match_index: m.start() + start_index,
                remove: true,
                skip_from_title: false,
            });
        }
        None
    }));

    parser.add_handler(Handler::from_regex(
        "languages",
        |t| &mut t.languages,
        Regex::case_insensitive(r"\b(temporadas?|completa)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Spanish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "complete",
        |t| &mut t.complete,
        Regex::case_insensitive(
            r"(?:\bthe\W)?(?:\bcomplete|collection|dvd)?\b[ .]?\bbox[ .-]?set\b",
        )
            .unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "complete",
        |t| &mut t.complete,
        Regex::case_insensitive(
            r"(?:\bthe\W)?(?:\bcomplete|collection|dvd)?\b[ .]?\bmini[ .-]?series\b",
        )
            .unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "complete",
        |t| &mut t.complete,
        Regex::case_insensitive(r"(?:\bthe\W)?(?:\bcomplete|full|all)\b.*\b(?:series|seasons|collection|episodes|set|pack|movies)\b")
            .unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "complete",
        |t| &mut t.complete,
        Regex::case_insensitive(r"\b(?:series|seasons|movies?)\b.*\b(?:complete|collection)\b")
            .unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "complete",
        |t| &mut t.complete,
        Regex::case_insensitive(r"(?:\bthe\W)?\bultimate\b[ .]\bcollection\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "complete",
        |t| &mut t.complete,
        Regex::case_insensitive(r"\bcollection\b.*\b(?:set|pack|movies)\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "complete",
        |t| &mut t.complete,
        Regex::case_insensitive(r"\bcollection\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            skip_from_title: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "complete",
        |t| &mut t.complete,
        Regex::case_insensitive(
            r"duology|trilogy|quadr[oi]logy|tetralogy|pentalogy|hexalogy|heptalogy|anthology",
        )
            .unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "complete",
        |t| &mut t.complete,
        Regex::case_insensitive(r"\bcompleta\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "complete",
        |t| &mut t.complete,
        Regex::case_insensitive(r"\bsaga\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(
            r"(?:complete\W|seasons?\W|\W|^)((?:s\d{1,2}[., +/\\&-]+)+s\d{1,2}\b)",
        )
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"(?:complete\W|seasons?\W|\W|^)[([]?(s\d{2,}-\d{2,}\b)[)\]]?")
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"(?:complete\W|seasons?\W|\W|^)[([]?(s[1-9]-[2-9])[)\]]?")
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"\d+ª(?:.+)?(?:a.?)?\d+ª(?:(?:.+)?(?:temporadas?))").unwrap(),
        transforms::range_func,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(
            r"(?:(?:\bthe\W)?\bcomplete\W)?(?:seasons?|[Сс]езони?|temporadas?)[. ]?[-:]?[. ]?[([]?((?:\d{1,2}[., /\\&]+)+\d{1,2}\b)[)\]]?",
        )
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(
            r"(?:(?:\bthe\W)?\bcomplete\W)?(?:seasons?|[Сс]езони?|temporadas?)[. ]?[-:]?[. ]?[([]?((?:\d{1,2}[.-]+)+[1-9]\d?\b)[)\]]?",
        )
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"(?:(?:\bthe\W)?\bcomplete\W)?season[. ]?[([]?((?:\d{1,2}[. -]+)+[1-9]\d?\b)[)\]]?(?!.*\.\w{2,4}$)")
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"(?:(?:\bthe\W)?\bcomplete\W)?\bseasons?\b[. -]?(\d{1,2}[. -]?(?:to|thru|and|\+|:)[. -]?\d{1,2})\b")
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"(?:(?:\bthe\W)?\bcomplete\W)?(?:saison|seizoen|season|series|temp(?:orada)?):?[. ]?(\d{1,2})\b").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"(\d{1,2})(?:-?й)?[. _]?(?:[Сс]езон|sez(?:on)?)(?:\W?\D|$)")
            .unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"[Сс]езон:?[. _]?№?(\d{1,2})(?!\d)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"(?:\D|^)(\d{1,2})Â?[°ºªa]?[. ]*temporada").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"t(\d{1,3})(?:[ex]+|$)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"(?:(?:\bthe\W)?\bcomplete)?s(\d{1,3})(?:[\Wex]|\d{2}\b|$)")
            .unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions {
            remove: false,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(
            r"(?:(?:\bthe\W)?\bcomplete\W)?(?:\W|^)(\d{1,2})[. ]?(?:st|nd|rd|th)[. ]*season",
        )
            .unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"(?<=S)\d{2}(?=E\d+)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"(?:\D|^)(\d{1,2})[xх]\d{1,3}(?:\D|$)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"\bSn([1-9])(?:\D|$)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"[[(](\d{1,2})\.\d{1,3}[)\]]").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"-\s?(\d{1,2})\.\d{2,3}\s?-").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"(?:^|\/)(\d{1,2})-\d{2}\b(?!-\d)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"[^\w-](\d{1,2})-\d{2}(?=\.\w{2,4}$)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(
            r"(?<!\bEp?(?:isode)? ?\d+\b.*)\b(\d{2})[ ._]\d{2}(?:.F)?\.\w{2,4}$",
        )
            .unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"\bEp(?:isode)?\W+(\d{1,2})\.\d{1,3}\b").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"\bSeasons?\b.*\b(\d{1,2}-\d{1,2})\b").unwrap(),
        transforms::range_func,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "seasons",
        |t| &mut t.seasons,
        Regex::case_insensitive(r"(?:\W|^)(\d{1,2})(?:e|ep)\d{1,3}(?:\W|$)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::from_regex(
        "episodes",
        |t| &mut t.episodes,
        Regex::case_insensitive(
            r"(?:[\W\d]|^)e[ .]?[([]?(\d{1,3}(?:[ .-]*(?:[&+]|e){1,2}[ .]?\d{1,3})+)(?:\W|$)",
        )
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |t| &mut t.episodes,
        Regex::case_insensitive(
            r"(?:[\W\d]|^)ep[ .]?[([]?(\d{1,3}(?:[ .-]*(?:[&+]|ep){1,2}[ .]?\d{1,3})+)(?:\W|$)",
        )
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |t| &mut t.episodes,
        Regex::case_insensitive(
            r"(?:[\W\d]|^)\d+[xх][ .]?[([]?(\d{1,3}(?:[ .]?[xх][ .]?\d{1,3})+)(?:\W|$)",
        )
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |t| &mut t.episodes,
        Regex::case_insensitive(r"(?:[\W\d]|^)(?:episodes?|[Сс]ерии:?)[ .]?[([]?(\d{1,3}(?:[ .+]*[&+][ .]?\d{1,3})+)(?:\W|$)").unwrap(),
        transforms::range_func,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |t| &mut t.episodes,
        Regex::case_insensitive(r"[([]?(?:\D|^)(\d{1,3}[ .]?ao[ .]?\d{1,3})[)\]]?(?:\W|$)")
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |t| &mut t.episodes,
        Regex::case_insensitive(r"(?:[\W\d]|^)(?:e|eps?|episodes?|[Сс]ерии:?|\d+[xх])[ .]*[([]?(\d{1,3}(?:-\d{1,3})+)(?:\W|$)").unwrap(),
        transforms::range_func,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |t| &mut t.episodes,
        Regex::case_insensitive(
            r"[st]\d{1,2}[. ]?[xх-]?[. ]?(?:e|x|х|ep|-|\.)[. ]?(\d{1,4})(?:[abc]|v0?[1-4]|\D|$)",
        )
            .unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |t| &mut t.episodes,
        Regex::case_insensitive(r"\b[st]\d{2}(\d{2})\b").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |t| &mut t.episodes,
        Regex::case_insensitive(r"(?:\W|^)(\d{1,3}(?:[ .]*~[ .]*\d{1,3})+)(?:\W|$)").unwrap(),
        transforms::range_func,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::case_insensitive(r"-\s(\d{1,3}[ .]*-[ .]*\d{1,3})(?!-\d)(?:\W|$)").unwrap(),
        transforms::range_func,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::case_insensitive(r"s\d{1,2}\s?\((\d{1,3}[ .]*-[ .]*\d{1,3})\)").unwrap(),
        transforms::range_func,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::new(r"(?:^|\/)\d{1,2}-(\d{2})\b(?!-\d)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::new(r"(?<!\d-)\b\d{1,2}-(\d{2})(?=\.\w{2,4}$)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |t| &mut t.episodes,
        Regex::case_insensitive(r"(?<=^\[.+].+)[. ]+-[. ]+(\d{1,4})[. ]+(?=\W)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |t| &mut t.episodes,
        Regex::case_insensitive(r"(?<!(?:seasons?|[Сс]езони?)\W*)(?:[ .([-]|^)(\d{1,3}(?:[ .]?[,&+~][ .]?\d{1,3})+)(?:[ .)\]-]|$)")
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |t| &mut t.episodes,
        Regex::case_insensitive(
            r"(?<!(?:seasons?|[Сс]езони?)\W*)(?:[ .([-]|^)(\d{1,3}(?:-\d{1,3})+)(?:[ .)(\]]|-\D|$)",
        )
            .unwrap(),
        transforms::range_func,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::case_insensitive(r"\bEp(?:isode)?\W+\d{1,2}\.(\d{1,3})\b").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::case_insensitive(r"(?:\b[ée]p?(?:isode)?|[Ээ]пизод|[Сс]ер(?:ии|ия|\.)?|cap(?:itulo)?|epis[oó]dio)[. ]?[-:#№]?[. ]?(\d{1,4})(?:[abc]|v0?[1-4]|\W|$)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::case_insensitive(
            r"\b(\d{1,3})(?:-?я)?[ ._-]*(?:ser(?:i?[iyj]a|\b)|[Сс]ер(?:ии|ия|\.)?)",
        )
            .unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::case_insensitive(r"(?:\D|^)\d{1,2}[. ]?[xх][. ]?(\d{1,3})(?:[abc]|v0?[1-4]|\D|$)")
            .unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::case_insensitive(r"(?<=S\d{2}E)\d+").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |t| &mut t.episodes,
        Regex::new(r"[[(]\d{1,2}\.(\d{1,3})[)\]]").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::new(r"\b[Ss]\d{1,2}[ .](\d{1,2})\b").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::new(r"-\s?\d{1,2}\.(\d{2,3})\s?-").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::case_insensitive(r"(?<=\D|^)(\d{1,3})[. ]?(?:of|из|iz)[. ]?\d{1,3}(?=\D|$)")
            .unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::new(r"\b\d{2}[ ._-](\d{2})(?:.F)?\.\w{2,4}$").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::new(r"(?<!^)\[(\d{2,3})](?!(?:\.\w{2,4})?$)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::case_insensitive(r"(\d+)(?=.?\[([A-Z0-9]{8})])").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::case_insensitive(r"(?<![xh])\b264\b|\b265\b").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::case_insensitive(r"(?<!\bMovie\s-\s)(?<=\s-\s)\d+(?=\s[-(\s])").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "episodes",
        |r| &mut r.episodes,
        Regex::case_insensitive(r"(?:\W|^)(?:\d+)?(?:e|ep)(\d{1,3})(?:\W|$)").unwrap(),
        |v, _| Some(vec![v.parse().ok()?]),
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::new("episodes", |context| {
        if context.result.episodes.is_empty() {
            let start_indexes = [
                context.matched.get("year").map(|m| m.match_index),
                context.matched.get("seasons").map(|m| m.match_index),
            ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            let end_indexes = [
                context.matched.get("resolution").map(|m| m.match_index),
                context.matched.get("quality").map(|m| m.match_index),
                context.matched.get("codec").map(|m| m.match_index),
                context.matched.get("audio").map(|m| m.match_index),
                Some(context.title.len()),
            ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();

            let start_index = start_indexes.iter().min().copied().unwrap_or(0);
            let end_index = end_indexes.iter().min().copied().unwrap_or(context.title.len());
            let start_index = min(start_index, end_index);

            // Ensure indices are at char boundaries
            let s = &context.title;
            let safe_start = if s.is_char_boundary(start_index) { start_index } else { s.len() };
            let safe_end = if s.is_char_boundary(end_index) { end_index } else { s.len() };

            let beginning_title = &s[..safe_end];
            let middle_title = &s[safe_start..safe_end];

            if let Some(m) = EPISODE_RE1
                .find_str(beginning_title)
                .or_else(|| EPISODE_RE2.find_str(middle_title))
            {
                let episode_str = m.group(1).unwrap().as_str();
                let episode_numbers: Vec<i32> = EPISODE_RE3
                    .find_iter_str(episode_str)
                    .filter_map(|m| m.as_str().parse().ok())
                    .collect();

                if !episode_numbers.is_empty() {
                    context.result.episodes = episode_numbers;
                    // Use .find on the full string, but ensure char boundary
                    if let Some(byte_idx) = s.find(m.as_str()) {
                        if s.is_char_boundary(byte_idx) {
                            return Some(HandlerResult {
                                raw_match: m.as_str().to_string(),
                                match_index: byte_idx,
                                remove: false,
                                skip_from_title: false,
                            });
                        }
                    }
                }
            }
        }
        None
    }));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bengl?(?:sub[A-Z]*)?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::English),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\beng?sub[A-Z]*\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::English),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bing(?:l[eéê]s)?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::English),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\besub\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::English),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\benglish\W+(?:subs?|sdh|hi)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::English),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\beng?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::English),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\benglish?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::English),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:JP|JAP|JPN)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Japanese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(japanese|japon[eê]s)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Japanese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:KOR|kor[ .-]?sub)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Korean),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(korean|coreano)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Korean),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:traditional\W*chinese|chinese\W*traditional)(?:\Wchi)?\b")
            .unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Chinese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bzh-hant\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Chinese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:mand[ae]rin|ch[sn])\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Chinese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"(?<!shang-?)\bCH(?:I|T)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Chinese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(chinese|chin[eê]s)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Chinese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bzh-hans\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Chinese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bFR(?:ench|a|e|anc[eê]s)?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::French),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(VOST(?:FR?|A)?)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::French),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(VF[FQIB2]?|(TRUE|SUB)?.?FRENCH|(VOST)?FR2?)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::French),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bspanish\W?latin|american\W*(?:spa|esp?)").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::LatinAmericanSpanish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:\bla\b.+(?:cia\b))").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Spanish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:audio.)?lat(?:in?|ino)?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::LatinAmericanSpanish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:audio.)?(?:ESP?|spa|(en[ .]+)?espa[nñ]ola?|castellano)\b")
            .unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Spanish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bes(?=[ .,/-]+(?:[A-Z]{2}[ .,/-]+){2,})\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Spanish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?<=[ .,/-]+(?:[A-Z]{2}[ .,/-]+){2,})es\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Spanish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?<=[ .,/-]+[A-Z]{2}[ .,/-]+)es(?=[ .,/-]+[A-Z]{2}[ .,/-]+)\b")
            .unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Spanish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bes(?=\.(?:ass|ssa|srt|sub|idx)$)").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Spanish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bspanish\W+subs?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Spanish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(spanish|espanhol)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Spanish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:p[rt]|en|port)[. (\\/-]*BR\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Portuguese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bbr(?:a|azil|azilian)\W+(?:pt|por)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Portuguese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(
            r"\b(?:leg(?:endado|endas?)?|dub(?:lado)?|portugu[eèê]se?)[. -]*BR\b",
        )
            .unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Portuguese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bleg(?:endado|endas?)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Portuguese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bportugu[eèê]s[ea]?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Portuguese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bPT[. -]*(?:PT|ENG?|sub(?:s|titles?))\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Portuguese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bpt(?=\.(?:ass|ssa|srt|sub|idx)$)").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Portuguese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bpor\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Portuguese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b-?ITA\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Italian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::new(r"\b(?<!w{3}\.\w+\.)IT(?=[ .,/-]+(?:[a-zA-Z]{2}[ .,/-]+){2,})\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Italian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bit(?=\.(?:ass|ssa|srt|sub|idx)$)").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Italian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bitaliano?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Italian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bgreek[ .-]*(?:audio|lang(?:uage)?|subs?(?:titles?)?)?\b")
            .unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Greek),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:GER|DEU)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::German),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bde(?=[ .,/-]+(?:[A-Z]{2}[ .,/-]+){2,})\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::German),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?<=[ .,/-]+(?:[A-Z]{2}[ .,/-]+){2,})de\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::German),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?<=[ .,/-]+[A-Z]{2}[ .,/-]+)de(?=[ .,/-]+[A-Z]{2}[ .,/-]+)\b")
            .unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::German),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bde(?=\.(?:ass|ssa|srt|sub|idx)$)").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::German),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(german|alem[aã]o)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::German),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bRUS?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Russian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(russian|russo)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Russian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bUKR\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Ukrainian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bukrainian\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Ukrainian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bhin(?:di)?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Hindi),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:(?<!w{3}\.\w+\.)tel(?!\W*aviv)|telugu)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Telugu),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bt[aâ]m(?:il)?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Tamil),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:(?<!w{3}\.\w+\.)MAL(?:ay)?|malayalam)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Malayalam),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:(?<!w{3}\.\w+\.)KAN(?:nada)?|kannada)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Kannada),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:(?<!w{3}\.\w+\.)MAR(?:a(?:thi)?)?|marathi)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Marathi),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:(?<!w{3}\.\w+\.)GUJ(?:arati)?|gujarati)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Gujarati),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:(?<!w{3}\.\w+\.)PUN(?:jabi)?|punjabi)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Punjabi),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:(?<!w{3}\.\w+\.)BEN(?!.\bThe|and|of\b)(?:gali)?|bengali)\b")
            .unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Bengali),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::new(r"\b(?<!YTS\.)LT\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Lithuanian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\blithuanian\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Lithuanian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\blatvian\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Latvian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bestonian\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Estonian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:(?<!w{3}\.\w+\.)PL|pol)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Polish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(polish|polon[eê]s|polaco)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Polish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bCZ[EH]?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Czech),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bczech\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Czech),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bslo(?:vak|vakian|subs|[\]_)]?\.\w{2,4}$)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Slovak),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::new(r"\bHU\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Hungarian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bHUN(?:garian)?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Hungarian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bROM(?:anian)?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Romanian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bRO(?=[ .,/-]*(?:[A-Z]{2}[ .,/-]+)*sub)").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Romanian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bbul(?:garian)?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Bulgarian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:srp|serbian)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Serbian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:HRV|croatian)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Croatian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bHR(?=[ .,/-]*(?:[A-Z]{2}[ .,/-]+)*sub)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Croatian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bslovenian\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Slovenian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:(?<!w{3}\.\w+\.)NL|dut|holand[eê]s)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Dutch),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bdutch\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Dutch),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bflemish\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Dutch),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:DK|danska|dansub|nordic)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Danish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(danish|dinamarqu[eê]s)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Danish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bdan\b(?=.*\.(?:srt|vtt|ssa|ass|sub|idx)$)").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Danish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:(?<!w{3}\.\w+\.)FI|finsk|finsub|nordic)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Finnish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bfinnish\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Finnish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:(?<!w{3}\.\w+\.)SE|swe|swesubs?|sv(?:ensk)?|nordic)\b")
            .unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Swedish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(swedish|sueco)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Swedish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:NOR|norsk|norsub|nordic)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Norwegian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(
            r"\b(norwegian|noruegu[eê]s|bokm[aå]l|nob|nor(?=[\]_)]?\.\w{2,4}$))\b",
        )
            .unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Norwegian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:arabic|[aá]rabe|ara)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Arabic),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\barab.*(?:audio|lang(?:uage)?|sub(?:s|titles?)?)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Arabic),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bar(?=\.(?:ass|ssa|srt|sub|idx)$)").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Arabic),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:turkish|tur(?:co)?)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Turkish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(TİVİBU|tivibu|bitturk(.net)?|turktorrent)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Turkish),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bvietnamese\b|\bvie(?=[\]_)]?\.\w{2,4}$)").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Vietnamese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bind(?:onesian)?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Indonesian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(thai|tailand[eê]s)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Thai),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::new(r"\b(THA|tha)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Thai),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(?:malay|may(?=[\]_)]?\.\w{2,4}$)|(?<=subs?\([a-z,]+)may)\b")
            .unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Malay),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_if_first: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\bheb(?:rew|raico)?\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Hebrew),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"\b(persian|persa)\b").unwrap(),
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Persian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\u3040-\u30ff]+").unwrap(), // japanese
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Japanese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\u3400-\u4dbf]+").unwrap(), // chinese
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Chinese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\u4e00-\u9fff]+").unwrap(), // chinese
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Chinese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\uf900-\ufaff]+").unwrap(), // chinese
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Chinese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\uff66-\uff9f]+").unwrap(), // japanese
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Japanese),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\u0400-\u04ff]+").unwrap(), // russian
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Russian),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\u0600-\u06ff]+").unwrap(), // arabic
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Arabic),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\u0750-\u077f]+").unwrap(), // arabic
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Arabic),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\u0c80-\u0cff]+").unwrap(), // kannada
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Kannada),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\u0d00-\u0d7f]+").unwrap(), // malayalam
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Malayalam),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\u0e00-\u0e7f]+").unwrap(), // thai
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Thai),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\u0900-\u097f]+").unwrap(), // hindi
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Hindi),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\u0980-\u09ff]+").unwrap(), // bengali
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Bengali),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "languages",
        |r| &mut r.languages,
        Regex::case_insensitive(r"[\u0a00-\u0a7f]+").unwrap(), // gujarati
        transforms::chain_transforms(
            transforms::replace_with_value(Language::Gujarati),
            transforms::uniq_concat,
        ),
        RegexHandlerOptions {
            skip_from_title: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::new("languages", |context| {
        if !context
            .result
            .languages
            .iter()
            .any(|lang| lang == &Language::Portuguese || lang == &Language::Spanish)
        {
            // Checking if episode naming convention suggests Portuguese language
            if (context
                .matched
                .get("episodes")
                .map(|e| &e.raw_match)
                .map(|raw| PT_LANG_RE1.contains_match(raw))
                .unwrap_or(false))
                || PT_LANG_RE2.contains_match(&context.title)
            {
                context.result.languages.push(Language::Portuguese);
            }
        }
        None
    }));

    parser.add_handler(Handler::from_regex(
        "subbed",
        |r| &mut r.subbed,
        Regex::case_insensitive(r"\b(?:Official.*?|Dual-?)?sub(s|bed)?\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "subbed",
        |r| &mut r.subbed,
        Regex::case_insensitive(r"\bmulti(?:ple)?[ .-]*(?:su?$|sub\w*|dub\w*)\b|msub").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            skip_if_already_found: false,
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "dubbed",
        |r| &mut r.dubbed,
        Regex::case_insensitive(r"\bmulti(?:ple)?[ .-]*(?:lang(?:uages?)?|audio|VF2)?\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "dubbed",
        |r| &mut r.dubbed,
        Regex::case_insensitive(r"\btri(?:ple)?[ .-]*(?:audio|dub\w*)\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "dubbed",
        |r| &mut r.dubbed,
        Regex::case_insensitive(r"\bdual[ .-]*(?:au?$|[aá]udio|line)\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "dubbed",
        |r| &mut r.dubbed,
        Regex::case_insensitive(r"\bdual\b(?![ .-]*sub)").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            skip_if_already_found: false,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "dubbed",
        |r| &mut r.dubbed,
        Regex::case_insensitive(r"\b(fan\s?dub)\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            skip_from_title: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "dubbed",
        |r| &mut r.dubbed,
        Regex::case_insensitive(r"\b(Fan.*)?(?:DUBBED|dublado|dubbing|DUBS?)\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "dubbed",
        |r| &mut r.dubbed,
        Regex::case_insensitive(
            r"\b(?!.*\bsub(s|bed)?\b)([ _\-\[(\.])?(dual|multi)([ _\-\[(\.])?(audio)?\b",
        )
            .unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "dubbed",
        |r| &mut r.dubbed,
        Regex::case_insensitive(
            r"\b(JAP?(anese)?|ZH)\+ENG?(lish)?|ENG?(lish)?\+(JAP?(anese)?|ZH)\b",
        )
            .unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::new("group", |context| {
        let Some(group_matched) = context.matched.get("group") else {
            return None;
        };
        if group_matched.raw_match.starts_with('[') && group_matched.raw_match.ends_with(']') {
            let end_index = group_matched.match_index + group_matched.raw_match.len();

            if context
                .matched
                .iter()
                .any(|(key, value)| key != "group" && value.match_index < end_index)
            {
                context.result.group = None; // remove group again
            }
        }
        None
    }));

    parser.add_handler(Handler::from_regex(
        "3d",
        |r| &mut r.is_3d,
        Regex::case_insensitive(r"(?<=\b[12]\d{3}\b).*\b(3d|sbs|half[ .-]ou|half[ .-]sbs)\b")
            .unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            skip_if_first: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "3d",
        |r| &mut r.is_3d,
        Regex::case_insensitive(r"\b((Half.)?SBS|HSBS)\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            skip_if_first: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "3d",
        |r| &mut r.is_3d,
        Regex::case_insensitive(r"\bBluRay3D\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            skip_if_first: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "3d",
        |r| &mut r.is_3d,
        Regex::case_insensitive(r"\bBD3D\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            skip_if_first: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "3d",
        |r| &mut r.is_3d,
        Regex::case_insensitive(r"\b3D\b").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: false,
            skip_if_first: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "size",
        |r| &mut r.size,
        Regex::case_insensitive(r"\b(\d+(\.\d+)?\s?(MB|GB|TB))\b").unwrap(),
        transforms::identity,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "site",
        |r| &mut r.site,
        Regex::case_insensitive(r"\[([^\]]+\.[^\]]+)\](?=\.\w{2,4}$|\s)").unwrap(),
        transforms::value("$1"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "site",
        |r| &mut r.site,
        Regex::case_insensitive(r"\bwww\.\w*\.\w+\b").unwrap(),
        transforms::value("$1"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bATVP?\b").unwrap(),
        transforms::const_value(Network::AppleTV),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bAMZN\b").unwrap(),
        transforms::const_value(Network::Amazon),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bNF|Netflix\b").unwrap(),
        transforms::const_value(Network::Netflix),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bNICK(elodeon)?\b").unwrap(),
        transforms::const_value(Network::Nickelodeon),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bDSNY?P?\b").unwrap(),
        transforms::const_value(Network::Disney),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bH(MAX|BO)\b").unwrap(),
        transforms::const_value(Network::HBO),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bHULU\b").unwrap(),
        transforms::const_value(Network::Hulu),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bCBS\b").unwrap(),
        transforms::const_value(Network::CBS),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bNBC\b").unwrap(),
        transforms::const_value(Network::NBC),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bAMC\b").unwrap(),
        transforms::const_value(Network::AMC),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bPBS\b").unwrap(),
        transforms::const_value(Network::PBS),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\b(Crunchyroll|[. -]CR[. -])\b").unwrap(),
        transforms::const_value(Network::Crunchyroll),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bVICE\b").unwrap(),
        transforms::const_value(Network::VICE),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bSony\b").unwrap(),
        transforms::const_value(Network::Sony),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bHallmark\b").unwrap(),
        transforms::const_value(Network::Hallmark),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bAdult.?Swim\b").unwrap(),
        transforms::const_value(Network::AdultSwim),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "network",
        |r| &mut r.network,
        Regex::case_insensitive(r"\bAnimal.?Planet|ANPL\b").unwrap(),
        transforms::const_value(Network::AnimalPlanet),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "extension",
        |r| &mut r.extension,
        Regex::case_insensitive(r"\.(3g2|3gp|avi|flv|mkv|mk3d|mov|mp2|mp4|m4v|mpe|mpeg|mpg|mpv|webm|wmv|ogm|divx|ts|m2ts|iso|vob|sub|idx|ttxt|txt|smi|srt|ssa|ass|vtt|nfo|html)$").unwrap(),
        transforms::lowercase,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "audio",
        |r| &mut r.audio,
        Regex::case_insensitive(r"\bMP3\b").unwrap(),
        transforms::chain_transforms(transforms::replace_value("MP3"), transforms::uniq_concat),
        RegexHandlerOptions {
            remove: true,
            skip_if_already_found: false,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "group",
        |r| &mut r.group,
        Regex::new(r"\(([\w-]+)\)(?:$|\.\w{2,4}$)").unwrap(),
        transforms::identity,
        RegexHandlerOptions::default(),
    ));
    parser.add_handler(Handler::from_regex(
        "group",
        |r| &mut r.group,
        Regex::new(r"\b(INFLATE|DEFLATE)\b").unwrap(),
        transforms::value("$1"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "group",
        |r| &mut r.group,
        Regex::case_insensitive(r"\b(?:Erai-raws|Erai-raws\.com)\b").unwrap(),
        transforms::value("Erai-raws"),
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "group",
        |r| &mut r.group,
        Regex::new(r"^\[([^[\]]+)]").unwrap(),
        transforms::identity,
        RegexHandlerOptions::default(),
    ));

    parser.add_handler(Handler::new("group", |context| {
        if let Some(group) = &context.result.group {
            if group == "-" || group == "" {
                context.result.group = None; // remove this from groups
            }
        }
        None
    }));

    parser.add_handler(Handler::from_regex(
        "trash",
        |r| &mut r.trash,
        Regex::case_insensitive(r"acesse o original").unwrap(),
        transforms::true_if_found,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));

    parser.add_handler(Handler::from_regex(
        "title",
        |r| &mut r.title,
        Regex::case_insensitive(r"\b100[ .-]*years?[ .-]*quest\b").unwrap(),
        transforms::identity_non_optional,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
    parser.add_handler(Handler::from_regex(
        "title",
        |r| &mut r.title,
        Regex::case_insensitive(r"\b(?:INTEGRALE?|INTÉGRALE?|INTERNAL|HFR)\b").unwrap(),
        transforms::identity_non_optional,
        RegexHandlerOptions {
            remove: true,
            ..Default::default()
        },
    ));
}