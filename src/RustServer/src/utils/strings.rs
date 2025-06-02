use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref WHITESPACE_RE: Regex = Regex::new(r"\s+").unwrap();
}

pub fn normalize_title(text: &str) -> String {
    let cleaned = text
        .to_lowercase()
        .replace('&', " and ")
        .replace(':', "")
        .replace('?', "")
        .replace('!', "")
        .replace('-', " ")
        .replace('.', " ")
        .replace('_', " ")
        .replace('!', "")
        .replace('(', "")
        .replace(')', "")
        .replace('[', "")
        .replace(']', "")
        .replace('\'', "")
        .replace(',', "")
        .replace('“', "")
        .replace('”', "")
        .replace('’', "")
        .replace('–', " ")
        .replace('—', " ");

    WHITESPACE_RE.replace_all(&cleaned, " ").trim().to_string()
}
