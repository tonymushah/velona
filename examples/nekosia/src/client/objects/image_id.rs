use serde::Deserialize;

use crate::client::objects::{
    Rating,
    image::{
        ImageAnimeSource, ImageAttribution, ImageMetadatas, ImageObjectColors, ImageSource,
        ImageUrls,
    },
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RatingObject {
    pub rating: Rating,
    pub available: bool,
    pub nsfw: f32,
    pub questionable: f32,
    pub safe: f32,
    pub suggestive: f32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageObjectWithObjectRating {
    pub id: String,
    pub colors: ImageObjectColors,
    pub image: ImageUrls,
    pub metadata: ImageMetadatas,
    pub category: String,
    pub tags: Vec<String>,
    pub rating: RatingObject,
    pub anime: ImageAnimeSource,
    pub source: ImageSource,
    pub attribution: ImageAttribution,
}
