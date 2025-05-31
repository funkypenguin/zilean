use std::str::FromStr;

use chrono::NaiveDate;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref SANITIZER_REGEX: Regex = Regex::new(r"\W+").unwrap();
}

pub fn identity(value: &str, _: &Option<String>) -> Option<Option<String>> {
    Some(Some(value.to_string()))
}

pub fn identity_non_optional(value: &str, _: &String) -> Option<String> {
    Some(value.to_string())
}

pub fn uppercase(value: &str, _: &Option<String>) -> Option<Option<String>> {
    Some(Some(value.to_uppercase()))
}

pub fn lowercase(value: &str, _: &Option<String>) -> Option<Option<String>> {
    Some(Some(value.to_lowercase()))
}

pub fn true_if_found(value: &str, _: &bool) -> Option<bool> {
    if value.is_empty() { None } else { Some(true) }
}

pub fn parse<T: FromStr>(value: &str, _: &Option<T>) -> Option<Option<T>> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        match value.parse::<T>() {
            Ok(result) => Some(Some(result)),
            Err(_) => None,
        }
    }
}

/// note: `$1` is replaced with the input value
pub fn value(value: &'static str) -> impl Fn(&str, &Option<String>) -> Option<Option<String>> {
    move |input_value: &str, _| {
        let mut result = value.to_string();
        result = result.replace("$1", input_value);
        Some(Some(result))
    }
}

pub fn const_value<T: Clone>(value: T) -> impl Fn(&str, &Option<T>) -> Option<Option<T>> {
    move |_, _| -> Option<Option<T>> { Some(Some(value.clone())) }
}

/// note: `$1` is replaced with the input value
pub fn replace_value(value: &'static str) -> impl Fn(&str) -> String {
    move |input_value: &str| -> String {
        let mut result = value.to_string();
        result = result.replace("$1", input_value);
        result
    }
}

pub fn replace_with_value<T: Clone>(value: T) -> impl Fn(&str) -> T {
    move |_| -> T { value.clone() }
}

fn convert_months(date_str: &str) -> String {
    let mut result = date_str.to_string();

    lazy_static! {
        static ref MONTH_MAPPING: [(Regex, &'static str); 12] = [
            (Regex::new(r"(?i)\bJanu\b").unwrap(), "Jan"),
            (Regex::new(r"(?i)\bFebr\b").unwrap(), "Feb"),
            (Regex::new(r"(?i)\bMarc\b").unwrap(), "Mar"),
            (Regex::new(r"(?i)\bApri\b").unwrap(), "Apr"),
            (Regex::new(r"(?i)\bMay\b").unwrap(), "May"),
            (Regex::new(r"(?i)\bJune\b").unwrap(), "Jun"),
            (Regex::new(r"(?i)\bJuly\b").unwrap(), "Jul"),
            (Regex::new(r"(?i)\bAugu\b").unwrap(), "Aug"),
            (Regex::new(r"(?i)\bSept\b").unwrap(), "Sep"),
            (Regex::new(r"(?i)\bOcto\b").unwrap(), "Oct"),
            (Regex::new(r"(?i)\bNove\b").unwrap(), "Nov"),
            (Regex::new(r"(?i)\bDece\b").unwrap(), "Dec"),
        ];
    }

    // Replace each full month name with its shortened form
    for (month, shortened) in MONTH_MAPPING.iter() {
        result = month.replace_all(&result, *shortened).to_string();
    }

    result
}

pub fn date_from_format(
    format: &'static str,
) -> impl Fn(&str, &Option<String>) -> Option<Option<String>> {
    move |input_value: &str, _| {
        let sanitized = SANITIZER_REGEX
            .replace_all(input_value, " ")
            .trim()
            .to_string();
        let sanitized = convert_months(&sanitized);

        #[cfg(feature = "debug")]
        println!(
            "input_value: '{}' sanitized: '{}' for format: '{}'",
            input_value, sanitized, format
        );

        let date = NaiveDate::parse_from_str(&sanitized, format).ok()?;

        Some(Some(date.format("%Y-%m-%d").to_string())) // normalize to YYYY-MM-DD
    }
}

pub fn date_from_formats(
    formats: &'static [&'static str],
) -> impl Fn(&str, &Option<String>) -> Option<Option<String>> {
    let format_functions: Vec<_> = formats
        .iter()
        .map(|format| date_from_format(format))
        .collect();

    move |input_value: &str, existing| {
        for format_function in &format_functions {
            let result = format_function(input_value, existing);
            if result.is_some() {
                return result;
            }
        }
        None
    }
}

pub fn uniq_concat<T: Clone + PartialEq>(value: impl Into<T>, result: &Vec<T>) -> Option<Vec<T>> {
    let mut result = result.clone();
    let value: T = value.into();
    if result.contains(&value) {
        return Some(result);
    }
    result.push(value);
    Some(result)
}

pub fn resolution_transform(value: &str, _: &Option<String>) -> Option<Option<String>> {
    let input_value = value.to_lowercase();

    if input_value.contains("2160") || input_value.contains("4k") {
        return Some(Some("2160p".to_string()));
    } else if input_value.contains("1440") || input_value.contains("2k") {
        return Some(Some("1440p".to_string()));
    } else if input_value.contains("1080") {
        return Some(Some("1080p".to_string()));
    } else if input_value.contains("720") {
        return Some(Some("720p".to_string()));
    } else if input_value.contains("480") {
        return Some(Some("480p".to_string()));
    } else if input_value.contains("360") {
        return Some(Some("360p".to_string()));
    } else if input_value.contains("240") {
        return Some(Some("240p".to_string()));
    }
    None
}

lazy_static! {
    static ref RANGE_REGEX: Regex = Regex::new(r"\d+").unwrap();
}

pub fn range_func(value: &str, _: &Vec<i32>) -> Option<Vec<i32>> {
    let numbers: Vec<i32> = RANGE_REGEX
        .find_iter(value)
        .flat_map(|m| m.as_str().parse::<i32>())
        .collect();

    if numbers.len() == 2 && numbers[0] < numbers[1] {
        Some((numbers[0]..=numbers[1]).collect())
    } else if numbers.len() > 2 && numbers.windows(2).all(|w| w[0] + 1 == w[1]) {
        Some(numbers)
    } else if numbers.len() == 1 {
        Some(numbers)
    } else {
        None
    }
}

pub fn chain_transforms<T, F1, F2, R1>(
    transform1: F1,
    transform2: F2,
) -> impl Fn(&str, &T) -> Option<T>
where
    F1: Fn(&str) -> R1,
    F2: Fn(R1, &T) -> Option<T>,
    R1: Clone,
{
    move |value: &str, state: &T| {
        let intermediate = transform1(value);
        transform2(intermediate, state)
    }
}
