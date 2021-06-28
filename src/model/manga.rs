use serde_derive::{Deserialize, Serialize};
use strum_macros::EnumString;

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

impl MediaInfo {
    pub fn new(media_type: MediaType, title: String, publishers: Vec<PublisherInfo>, licensed_in_english: bool) -> Self {
        MediaInfo {
            media_type,
            title,
            publishers,
            licensed_in_english,
        }
    }
}

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
    pub fn new(publisher_type: PublisherType, name: String, vols: Option<usize>, status: Option<Status>) -> Self {
        PublisherInfo {
            publisher_type,
            name,
            vols,
            status,
        }
    }
}