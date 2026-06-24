use std::{collections::HashMap, time::Duration};

use regex::Regex;
use reqwest::{blocking::Client, redirect::Policy};
use serde::Deserialize;

use crate::error::{AppError, AppResult};

#[derive(Debug)]
pub struct GameMetadata {
    pub title: String,
    pub description: String,
    pub short_review: String,
    pub release_year: Option<i32>,
    pub source: String,
}

#[derive(Deserialize)]
struct SearchResponse {
    query: SearchQuery,
}

#[derive(Deserialize)]
struct SearchQuery {
    search: Vec<SearchHit>,
}

#[derive(Deserialize)]
struct SearchHit {
    title: String,
}

#[derive(Deserialize)]
struct PageResponse {
    query: PageQuery,
}

#[derive(Deserialize)]
struct PageQuery {
    pages: HashMap<String, WikiPage>,
}

#[derive(Deserialize)]
struct WikiPage {
    title: String,
    extract: Option<String>,
}

pub fn fetch(title: &str, system_name: &str) -> AppResult<Option<GameMetadata>> {
    let normalized = normalize_title(title);
    let client = Client::builder()
        .user_agent("RetroBox-Desktop/0.1 (local game library metadata)")
        .redirect(Policy::limited(3))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(http_error)?;
    let query = format!("intitle:\"{normalized}\" video game {system_name}");
    let search: SearchResponse = client
        .get("https://en.wikipedia.org/w/api.php")
        .query(&[
            ("action", "query"),
            ("format", "json"),
            ("list", "search"),
            ("srnamespace", "0"),
            ("srlimit", "5"),
            ("srsearch", query.as_str()),
        ])
        .send()
        .map_err(http_error)?
        .error_for_status()
        .map_err(http_error)?
        .json()
        .map_err(http_error)?;
    let normalized_lower = normalized.to_ascii_lowercase();
    let Some(hit) = search.query.search.into_iter().find(|hit| {
        let candidate = hit
            .title
            .trim_end_matches(" (video game)")
            .trim_end_matches(" (series)")
            .to_ascii_lowercase();
        candidate == normalized_lower
    }) else {
        return Ok(None);
    };
    let details: PageResponse = client
        .get("https://en.wikipedia.org/w/api.php")
        .query(&[
            ("action", "query"),
            ("format", "json"),
            ("prop", "extracts"),
            ("redirects", "1"),
            ("exintro", "1"),
            ("explaintext", "1"),
            ("titles", hit.title.as_str()),
        ])
        .send()
        .map_err(http_error)?
        .error_for_status()
        .map_err(http_error)?
        .json()
        .map_err(http_error)?;
    let Some(page) = details.query.pages.into_values().next() else {
        return Ok(None);
    };
    let extract = page.extract.unwrap_or_default();
    if extract.trim().is_empty() || !extract.to_ascii_lowercase().contains("video game") {
        return Ok(None);
    }
    let sentences = sentences(&extract);
    let description = truncate(
        &sentences
            .iter()
            .take(2)
            .cloned()
            .collect::<Vec<_>>()
            .join(" "),
        700,
    );
    let review_source = sentences
        .iter()
        .skip(2)
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    let short_review = truncate(
        if review_source.is_empty() {
            &description
        } else {
            &review_source
        },
        520,
    );
    let release_year = Regex::new(r"\b(19|20)\d{2}\b")
        .expect("valid year regex")
        .find(&extract)
        .and_then(|value| value.as_str().parse().ok());
    let mut source =
        url::Url::parse("https://en.wikipedia.org/wiki/").expect("valid Wikipedia base URL");
    source
        .path_segments_mut()
        .expect("Wikipedia URL can hold path segments")
        .push(&page.title.replace(' ', "_"));
    Ok(Some(GameMetadata {
        title: page.title.trim_end_matches(" (video game)").to_owned(),
        description,
        short_review,
        release_year,
        source: source.to_string(),
    }))
}

pub fn normalize_title(input: &str) -> String {
    let tags = Regex::new(r"(?i)\s*[\(\[].*?[\)\]]").expect("valid tag regex");
    let discs = Regex::new(r"(?i)\s*(disc|disk|cd|track)\s*\d+.*$").expect("valid disc regex");
    let without_tags = tags.replace_all(input, "");
    discs
        .replace(&without_tags, "")
        .replace(['_', '.'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn sentences(input: &str) -> Vec<String> {
    input
        .split_inclusive(['.', '!', '?'])
        .map(str::trim)
        .filter(|value| value.len() > 20)
        .map(str::to_owned)
        .collect()
}

fn truncate(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.trim().to_owned();
    }
    let mut value = input.chars().take(max_chars).collect::<String>();
    if let Some(index) = value.rfind(' ') {
        value.truncate(index);
    }
    value.push('…');
    value
}

fn http_error(error: reqwest::Error) -> AppError {
    AppError::InvalidInput(format!("Metadata HTTP chyba: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_dump_and_disc_tags() {
        assert_eq!(
            normalize_title("Metal Gear Solid (Europe) [Disc 1]"),
            "Metal Gear Solid"
        );
        assert_eq!(normalize_title("Super_Mario_World"), "Super Mario World");
    }
}
