use std::fmt::Display;
use serde_derive::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(strum_macros::Display)]
#[derive(EnumString, Debug, Serialize, Deserialize)]
pub enum MediaType {
    Novel,
    Manga,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaInfo {
    media_type: MediaType,
    pub(crate) title: String,
    pub(crate) publishers: Vec<PublisherInfo>,
    licensed_in_english: bool,
}

impl Display for MediaInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let publishers = self.publishers.iter().map(|p| {
            let maybe_vols = p.vols.map(|v| v.to_string()).or_else(|| Some("N/A".to_string())).unwrap();
            let maybe_status = p.status.clone().map(|s| s.to_string()).or_else(|| Some("N/A".to_string())).unwrap();
            format!("{} ({:?}) - Volumes: {} ({})", p.name, p.publisher_type, maybe_vols, maybe_status)
        }).collect::<Vec<String>>().join("\n");
        writeln!(f, "Title: {}\nMedia Type: {}\nPublishers:\n{}\nLicensed in English: {}",
                 self.title,
                 self.media_type,
                 publishers,
                 self.licensed_in_english
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub href: String
}

impl SearchResult {
    pub fn new(name: String, href: String) -> Self {
        SearchResult { name, href }
    }
}

impl MediaInfo {
    pub fn new(
        media_type: MediaType,
        title: String,
        publishers: Vec<PublisherInfo>,
        licensed_in_english: bool,
    ) -> Self {
        MediaInfo {
            media_type,
            title,
            publishers,
            licensed_in_english,
        }
    }
}

#[derive(strum_macros::Display)]
#[derive(EnumString, Debug, Serialize, Deserialize, Clone)]
pub enum Status {
    Complete,
    Ongoing,
    Hiatus,
}

#[derive(EnumString, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum PublisherType {
    #[strum(serialize = "Original Publisher")]
    Original,
    #[strum(serialize = "English Publisher")]
    English,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublisherInfo {
    pub publisher_type: PublisherType,
    pub name: String,
    pub vols: Option<usize>,
    pub status: Option<Status>,
}

impl PublisherInfo {
    pub fn new(
        publisher_type: PublisherType,
        name: String,
        vols: Option<usize>,
        status: Option<Status>,
    ) -> Self {
        PublisherInfo {
            publisher_type,
            name,
            vols,
            status,
        }
    }
}
