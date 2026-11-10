use color::DynamicColor;
use serde::{Deserialize, Deserializer};
use url::Url;

use crate::client::objects::Rating;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageObjectColors {
    #[serde(deserialize_with = "color_deser")]
    pub main: DynamicColor,
    #[serde(deserialize_with = "colors_deser")]
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
    pub profile: Option<Url>,
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
    pub metadata: ImageMetadatas,
    pub category: String,
    pub tags: Vec<String>,
    pub rating: Rating,
    pub anime: ImageAnimeSource,
    pub source: ImageSource,
    pub attribution: ImageAttribution,
}

fn color_deser<'de, D>(deserializer: D) -> Result<DynamicColor, D::Error>
where
    D: Deserializer<'de>,
{
    let hex = String::deserialize(deserializer)?;
    color::parse_color(&hex).map_err(serde::de::Error::custom)
}

fn colors_deser<'de, D>(deserializer: D) -> Result<Vec<DynamicColor>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<String>::deserialize(deserializer)?
        .into_iter()
        .map(|a| color::parse_color(&a))
        .collect::<Result<Vec<_>, color::ParseError>>()
        .map_err(serde::de::Error::custom)
}
