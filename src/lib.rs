use std::str::FromStr;

use scraper::{ElementRef, Html, Selector};
use selectors::Element;
use serde_derive::{Deserialize, Serialize};
use strum_macros::EnumString;
use std::iter::FromIterator;

#[derive(EnumString, Debug, Serialize, Deserialize)]
enum MediaType {
    Novel,
    Manga,
    Unknown
}

#[derive(Debug, Serialize, Deserialize)]
struct MediaInfo {
    media_type: MediaType,
    title: String,
    publishers: Vec<PublisherInfo>,
    licensed_in_english: bool
}

#[derive(EnumString, Debug, Serialize, Deserialize, Clone)]
enum Status {
    Complete,
    Ongoing,
    Hiatus,
}

#[derive(EnumString, Debug, Serialize, Deserialize)]
enum PublisherType {
    #[strum(serialize = "Original Publisher")]
    Original,
    #[strum(serialize = "English Publisher")]
    English,
}

#[derive(Debug, Serialize, Deserialize)]
struct PublisherInfo {
    publisher_type: PublisherType,
    name: String,
    vols: Option<usize>,
    status: Option<Status>,
}

fn get_media_info(page: &Html) -> MediaInfo {
    MediaInfo {
        media_type: get_media_type(page),
        title: get_title(page),
        publishers: get_publisher_info(page),
        licensed_in_english: get_licensed_status(page)
    }
}

fn get_media_type(page: &Html) -> MediaType {
    let text = get_value_of_block_with_text(page, r#"div.sCat > b"#.to_string(), Some("Type".to_string())).unwrap().parent_element().unwrap().next_sibling_element().unwrap().text().next().unwrap().replace("\n", "");
    MediaType::from_str(&text).unwrap_or(MediaType::Unknown)
}

fn get_licensed_status(page: &Html) -> bool {
    let text = get_value_of_block_with_text(page, r#"div.sCat > b"#.to_string(), Some("Licensed".to_string())).unwrap().parent_element().unwrap().next_sibling_element().unwrap().text().next().unwrap().replace("\n", "");
    match text.as_str() {
        "Yes" => true,
        "No" => false,
        _ => false
    }
}

fn get_volume_details(fragment: &str) -> (Option<usize>, Option<Status>) {
    let bounds: Vec<_> = fragment.chars().take_while(|c| c != &')').collect();
    let finals: String = String::from_iter(bounds);
    let cleaned = finals.replace("(", "").replace(")", "");
    let splits = cleaned.split_ascii_whitespace().map(|s| s.to_owned()).collect::<Vec<String>>();
    let status = splits.last().and_then(|status_text| {
        Status::from_str(status_text).ok()
    });
    let vols = splits.first().and_then(|x| str::parse::<usize>(x).ok());
    (vols, status)
}

fn get_title(page: &Html) -> String {
    let selector_raw = r#"span.releasestitle"#;
    let selector = Selector::parse(selector_raw).unwrap();
    let matches: Vec<ElementRef> = page.select(&selector).collect();
    matches.first().unwrap().text().next().unwrap().to_string()
}

fn get_value_of_block_with_text(page: &Html, pattern: String, text_match: Option<String>) -> Option<ElementRef> {
    let selector = Selector::parse(&pattern).unwrap();
    let selector_matches: Vec<ElementRef> = page.select(&selector).collect();
    let matches: Vec<_> = match text_match {
        None => {
            selector_matches.to_vec()
        }
        Some(t) => {
            selector_matches.iter().cloned().filter(|e| {
                let element = e.text().next().map(|x| x.to_string());
                match element {
                    Some(text) => {
                        text.contains(&t)
                    }
                    _ => false
                }
            }).collect()
        }
    };

    let cloned = matches.clone();
    let first = cloned.first();
    first.cloned()
}

fn get_publisher_info(page: &Html) -> Vec<PublisherInfo> {
    let original_publisher_details = get_original_publisher_info(page);

    let selector_raw = r#"div.sContent > a[title="Publisher Info"]"#;
    let selector = Selector::parse(selector_raw).unwrap();
    let matches: Vec<ElementRef> = page.select(&selector).collect();
    matches.iter()
        .map(|e| {
            let publisher_type_text = &e.parent_element().unwrap().prev_sibling_element().unwrap().text().next().unwrap();
            let publisher_type: PublisherType = PublisherType::from_str(publisher_type_text).unwrap();
            let name = e.text().next().unwrap().to_string();

            let (vols, status) = match publisher_type {
                PublisherType::Original => original_publisher_details.clone(),
                PublisherType::English => get_volume_details(e.next_sibling().unwrap().value().as_text().unwrap())
            };


            PublisherInfo {
                name,
                vols,
                status,
                publisher_type,
            }
        }).collect::<Vec<_>>()
}

fn get_original_publisher_info(page: &Html) -> (Option<usize>, Option<Status>) {
    let serialization_status_selector_raw = r#"div.sCat > b"#;
    let serialization_status = get_value_of_block_with_text(page, serialization_status_selector_raw.to_string(), Some("Status".to_string()));
    serialization_status.map_or((None, None), |x| {
        x.parent_element().map_or((None, None), |y| y.next_sibling_element().map_or((None, None), |n| n.text().next().map_or((None, None), |status| {
            get_volume_details(status)
        })))
    })
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    const SKIP_BEAT: &str = include_str!("../test/static/Baka-Updates Manga - Skip Beat!.html");
    const BABY_STEPS: &str = include_str!("../test/static/Baka-Updates Manga - Baby Steps.html");
    const HUNTER_X_HUNTER: &str = include_str!("../test/static/Baka-Updates Manga - Hunter x Hunter.html");
    const HAGANAI_MANGA: &str = include_str!("../test/static/Baka-Updates Manga - Boku wa Tomodachi ga Sukunai.html");
    const HAGANAI_NOVEL: &str = include_str!("../test/static/Baka-Updates Manga - Boku wa Tomodachi ga Sukunai (Novel).html");

    #[test_case(SKIP_BEAT)]
    #[test_case(BABY_STEPS)]
    #[test_case(HUNTER_X_HUNTER)]
    #[test_case(HAGANAI_MANGA)]
    #[test_case(HAGANAI_NOVEL)]
    pub fn test_publisher_info(page: &str) {
        let html = Html::parse_document(page);
        let info = get_media_info(&html);
        println!("{:=^1$}", format!("Start {}", info.title), 30);
        assert!(!info.title.is_empty());
        println!("{}", serde_json::to_string_pretty(&info).unwrap());
        println!("{:=^1$}", format!("End {}", info.title), 30);
    }
}