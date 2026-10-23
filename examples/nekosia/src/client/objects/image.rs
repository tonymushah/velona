use color::DynamicColor;
use serde::Deserialize;
use url::Url;

use crate::client::objects::Rating;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageObjectColors {
    pub main: DynamicColor,
    pub palette: Vec<DynamicColor>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageUrl {
    pub url: Url,
    pub extension: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageUrls {
    pub original: ImageUrl,
    pub compressed: ImageUrl,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub extension: String,
    pub color_profile: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageMetadatas {
    pub original: ImageMetadata,
    pub compressed: ImageMetadata,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageAnimeSource {
    pub title: Option<String>,
    pub character: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageSource {
    pub url: Url,
    pub direct: Url,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageArtist {
    pub username: String,
    pub profile: Url,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageAttribution {
    pub artist: ImageArtist,
    pub copyright: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
    pub id: String,
    pub colors: ImageObjectColors,
    pub image: ImageUrls,
    pub metadata: String,
    pub tags: Vec<String>,
    pub rating: Rating,
    pub anime: ImageAnimeSource,
    pub source: ImageSource,
    pub attribution: ImageAttribution,
}
