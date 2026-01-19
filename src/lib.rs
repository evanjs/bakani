use std::iter::FromIterator;
use std::str::FromStr;

use reqwest::{Url, Client};
use scraper::{ElementRef, Html, Selector};
use selectors::Element;

use model::manga::*;

pub mod model;

const BAKA_MAIN_URL: &str = "https://www.mangaupdates.com";

fn get_media_info(page: &Html) -> MediaInfo {
    MediaInfo::new(
        get_media_type(page),
        get_title(page),
        get_publisher_info(page),
        get_licensed_status(page),
    )
}

fn parse_search_results(page: &Html) -> anyhow::Result<Vec<SearchResult>> {
    let selector_text = r#"main#mu-main > div > div:nth-of-type(2) > div > div:last-of-type > div div div > [title="Click for Series Info"]"#;
    let selector = Selector::parse(selector_text).unwrap();
    let selector_matches: Vec<ElementRef> = page.select(&selector).collect();
    let collected = selector_matches.iter().map(|e| {
        let href = e.value().attr("href").unwrap().to_string();
        let name = e.text().next().unwrap().to_string();
        SearchResult::new(name, href)
    }).collect::<Vec<_>>();
    Ok(collected)
}

fn get_media_type(page: &Html) -> MediaType {
    let text = get_value_of_block_with_text(
        page,
        r#"[data-cy="info-box-type-header"] > b"#.to_string(),
        Some("Type".to_string()),
    )
        .unwrap()
        .parent_element()
        .unwrap()
        .next_sibling_element()
        .unwrap()
        .text()
        .next()
        .unwrap()
        .replace("\n", "");
    MediaType::from_str(&text).unwrap_or(MediaType::Unknown)
}

fn get_licensed_status(page: &Html) -> bool {
    let text = get_value_of_block_with_text(
        page,
        r#"div[data-cy="info-box-licensed-header"] > b"#.to_string(),
        Some("Licensed".to_string()),
    )
        .unwrap()
        .parent_element()
        .unwrap()
        .next_sibling_element()
        .unwrap()
        .text()
        .next()
        .unwrap()
        .replace("\n", "");
    match text.as_str() {
        "Yes" => true,
        "No" => false,
        _ => false,
    }
}

fn get_volume_details(fragment: &str) -> (Option<usize>, Option<Status>) {
    let bounds: Vec<_> = fragment.chars().take_while(|c| c != &')').collect();
    let finals: String = String::from_iter(bounds);
    let cleaned = finals.replace("(", "").replace(")", "");
    let splits = cleaned
        .split_ascii_whitespace()
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();
    let status = splits
        .last()
        .and_then(|status_text| Status::from_str(status_text).ok());
    let vols = splits.first().and_then(|x| str::parse::<usize>(x).ok());
    (vols, status)
}

fn get_title(page: &Html) -> String {
    let selector_raw = r#".releasestitle, tabletitle"#;
    let selector = Selector::parse(selector_raw).unwrap();
    let matches: Vec<ElementRef> = page.select(&selector).collect();
    matches.first().unwrap().text().next().unwrap().to_string()
}

pub async fn search_for_baka_title(query: String) -> anyhow::Result<Vec<SearchResult>> {
    let search_results = search_baka_title(&query).await?;
    let html = Html::parse_document(search_results.as_str());
    parse_search_results(&html)
}

pub async fn search_and_get_baka_entry(query: &str) -> anyhow::Result<MediaInfo> {
    let search_results = search_for_baka_title(query.to_string()).await?;
    let first = search_results.first().unwrap();
    let baka_entry = get_baka_entry_from_url(&first.href).await?;
    let html = Html::parse_document(baka_entry.as_str());
    let info = get_media_info(&html);

    Ok(info)
}

pub async fn get_baka_entry(id: &str) -> reqwest::Result<String> {
    let mut url = Url::parse(&format!("{}", BAKA_MAIN_URL)).unwrap();
    url.path_segments_mut()
        .unwrap()
        .push("series")
        .push(id);
    let client = reqwest::Client::new();
    client.get(url).query(&[("id", id)]).send().await?.text().await
}

pub async fn get_baka_entry_from_url(url: &str) -> reqwest::Result<String> {
    let client = reqwest::Client::new();
    client.get(url).send().await?.text().await
}

pub async fn request_baka_title_post(query: String) -> reqwest::Result<String> {
    let client = Client::new();
    let url = format!("{}/series.html", BAKA_MAIN_URL);
    let request = client.post(url).form(&[("search", query)]);
    request.send().await?.text().await
}

pub async fn search_baka_title(query: &str) -> reqwest::Result<String> {
    let client = Client::new();
    let mut url = Url::parse(&format!("{}", BAKA_MAIN_URL)).unwrap();
    url.path_segments_mut()
        .unwrap()
        .push("site")
        .push("search")
        .push("result");
    let request = client.get(url).query(&[("search", query)]);
    request.send().await?.text().await
}

fn get_value_of_block_with_text(
    page: &Html,
    pattern: String,
    text_match: Option<String>,
) -> Option<ElementRef> {
    let selector = Selector::parse(&pattern).unwrap();
    let selector_matches: Vec<ElementRef> = page.select(&selector).collect();
    let matches: Vec<_> = match text_match {
        None => selector_matches.to_vec(),
        Some(t) => selector_matches
            .iter()
            .cloned()
            .filter(|e| {
                let element = e.text().next().map(|x| x.to_string());
                match element {
                    Some(text) => text.contains(&t),
                    _ => false,
                }
            })
            .collect(),
    };

    let cloned = matches.clone();
    let first = cloned.first();
    first.cloned()
}

fn get_publisher_info(page: &Html) -> Vec<PublisherInfo> {
    let serialization_status = get_serialization_status(page);

    let selector_raw = r#"div[data-cy="info-box-original_publisher"] > div"#;
    let selector = Selector::parse(selector_raw).unwrap();
    let matches: Vec<ElementRef> = page.select(&selector).collect();
    matches
        .iter()
        .map(|e| {
            let publisher_type_text = &e
                .parent_element()
                .unwrap()
                .prev_sibling_element()
                .unwrap()
                .text()
                .next()
                .unwrap();
            let publisher_type: PublisherType =
                PublisherType::from_str(publisher_type_text).unwrap();
            let name = e.text().next().unwrap().to_string();

            let (vols, status) = match publisher_type {
                PublisherType::Original => serialization_status.clone(),
                PublisherType::English => {
                    get_volume_details(e.next_sibling().unwrap().value().as_text().unwrap())
                }
            };

            PublisherInfo::new(publisher_type, name, vols, status)
        })
        .collect::<Vec<_>>()
}

fn get_serialization_status(page: &Html) -> (Option<usize>, Option<Status>) {
    let serialization_status_selector_raw = r#"div[data-cy="info-box-status-header"] > b"#;
    let serialization_status = get_value_of_block_with_text(
        page,
        serialization_status_selector_raw.to_string(),
        Some("Status".to_string()),
    );
    serialization_status.map_or((None, None), |x| {
        x.parent_element().map_or((None, None), |y| {
            y.next_sibling_element().map_or((None, None), |n| {
                n.text()
                    .next()
                    .map_or((None, None), |status| get_volume_details(status))
            })
        })
    })
}

#[cfg(test)]
mod tests {
    use std::ops::Index;
    use test_case::test_case;

    use super::*;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    const SKIP_BEAT: &str = include_str!("../test/static/Baka-Updates Manga - Skip Beat!.html");
    const BABY_STEPS: &str = include_str!("../test/static/Baka-Updates Manga - Baby Steps.html");
    const HUNTER_X_HUNTER: &str =
        include_str!("../test/static/Baka-Updates Manga - Hunter x Hunter.html");
    const HAGANAI_MANGA: &str =
        include_str!("../test/static/Baka-Updates Manga - Boku wa Tomodachi ga Sukunai.html");
    const HAGANAI_NOVEL: &str = include_str!(
        "../test/static/Baka-Updates Manga - Boku wa Tomodachi ga Sukunai (Novel).html"
    );
    const HIGEHIRO_SEARCH: &str = include_str!("../test/static/HigeHiro/Baka-Updates Manga - Series.html");
    const SKIP_BEAT_SEARCH: &str = include_str!("../test/static/SkipBeat/Baka-Updates Manga - Series.html");

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

    #[test_case(&"qlygvts", "Hakusensha"; "Skip Beat!")]
    pub fn test_get_entry(id: &str, publisher: &str) {
        let page = aw!(get_baka_entry(id)).unwrap();
        let html = Html::parse_document(page.as_str());
        let info = get_media_info(&html);
        let original_publishers = info
            .publishers
            .iter()
            .filter(|p| p.publisher_type == PublisherType::Original)
            .collect::<Vec<_>>();
        let first_original_publisher = original_publishers.first();
        let first_original_publisher = first_original_publisher
        .unwrap();
        assert_eq!(first_original_publisher.name, publisher);
        println!(
            "Validated Original Publisher for {}: {}",
            info.title, first_original_publisher.name
        );
    }

    #[test_case("Skip Beat!", &"qlygvts")]
    pub fn test_search_series(title: &str, id: &str) {
        let results = aw!(search_for_baka_title(title.to_string()));
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_ne!(results.is_empty(), true, "Results should not be empty.");
        let first = results.first().unwrap();
        let url = Url::parse(&first.href).unwrap();
        assert!(url.path_segments().is_some());
        let segments = url.path_segments().unwrap();
        let str_segments = segments.collect::<Vec<_>>();
        let series_path_actual = str_segments.index(0);
        let series_id_actual = str_segments.index(1);
        assert!(series_path_actual.eq(&"series"), "Input {:?} does not match expected value: {:#?}", series_path_actual, "series");
        assert!(series_id_actual.eq(&id), "Input {:?} does not match expected value: {:#?}", series_id_actual, id);

        let serialized = serde_json::to_string_pretty(&results).unwrap();
        println!("{}", serialized);
    }

    #[test_case(HIGEHIRO_SEARCH)]
    #[test_case(SKIP_BEAT_SEARCH)]
    pub fn test_parse_series_search_results(page: &str) {
        let html = Html::parse_document(page);
        let results = parse_search_results(&html);
        assert!(results.is_ok());
        let serialized = serde_json::to_string_pretty(&results.unwrap()).unwrap();
        println!("{}", serialized);
    }
}
