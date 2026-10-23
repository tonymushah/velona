use std::{future::Pending, num::NonZeroU8};

use arrayvec::ArrayVec;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::{
    endpoint::{Endpoint, handle_response_as_json},
    objects::{Rating, image::ImageObject},
};

type Tags20Array = ArrayVec<String, 20>;

/// This endpoint allows you to fetch random images from a selected category.
///
/// Check [https://nekosia.cat/documentation?page=endpoints#get-image-category] for more details
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRandomFromCategory {
    #[serde(skip_serializing)]
    category: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    count: Option<NonZeroU8>,

    #[serde(
        with = "serde_qs::helpers::comma_separated",
        skip_serializing_if = "ArrayVec::is_empty"
    )]
    additional_tags: Tags20Array,

    #[serde(
        with = "serde_qs::helpers::comma_separated",
        skip_serializing_if = "ArrayVec::is_empty"
    )]
    blacklisted_tags: ArrayVec<String, 20>,

    #[serde(skip_serializing_if = "Option::is_none")]
    rating: Option<Rating>,

    #[serde(skip_serializing_if = "Option::is_none")]
    session: Option<SessionTypeRepr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
#[error("The count maximum value (n > 20)")]
pub struct CountMaxError;

#[derive(Debug, thiserror::Error)]
#[error("The tag array is already full (tags > 20)")]
pub struct TagsArrayFull<V>(pub V);

impl GetRandomFromCategory {
    pub fn new<C>(category: C) -> Self
    where
        C: ToString,
    {
        Self {
            category: category.to_string(),
            count: None,
            additional_tags: Default::default(),
            blacklisted_tags: Default::default(),
            rating: None,
            session: None,
            id: None,
        }
    }
}

impl GetRandomFromCategory {
    /// `:category` - Check the available tags on [the Booru page](https://nekosia.cat/booru/tags).
    /// Each image is assigned to one main category.
    pub fn category(&mut self, category: String) -> &mut Self {
        self.category = category;
        self
    }
    /// `?count` (default: 1) - The number of images to fetch (default: 1, maximum: 20).
    /// The higher the number, the longer the server response time (in milliseconds).
    pub fn count(&mut self, count: NonZeroU8) -> Result<&mut Self, CountMaxError> {
        if count.get() > 20 {
            Err(CountMaxError)
        } else {
            self.count.replace(count);
            Ok(self)
        }
    }
    pub fn unset_count(&mut self) -> &mut Self {
        self.count.take();
        self
    }
    /// `&additionalTags` - Additional tags to include when searching for images, separated by commas (up to 20).
    pub fn additional_tags(&mut self, tags: Tags20Array) -> &mut Self {
        self.additional_tags = tags;
        self
    }
    pub fn additional_tag<T>(&mut self, tag: T) -> Result<&mut Self, TagsArrayFull<T>>
    where
        T: ToString,
    {
        if self.additional_tags.is_full() {
            Err(TagsArrayFull(tag))
        } else {
            self.additional_tags.push(tag.to_string());
            Ok(self)
        }
    }
    /// `&blacklistedTags` - Tags to exclude from the search results, separated by commas (up to 20).
    pub fn blacklisted_tags(&mut self, tags: Tags20Array) -> &mut Self {
        self.blacklisted_tags = tags;
        self
    }
    pub fn blacklisted_tag<T>(&mut self, tag: T) -> Result<&mut Self, TagsArrayFull<T>>
    where
        T: ToString,
    {
        if self.blacklisted_tags.is_full() {
            Err(TagsArrayFull(tag))
        } else {
            self.blacklisted_tags.push(tag.to_string());
            Ok(self)
        }
    }
    /// &rating: Filters images by content rating.
    ///
    /// See [Content Ratings](https://nekosia.cat/documentation?page=introduction#content-ratings) for more details.
    pub fn rating(&mut self, rating: Rating) -> &mut Self {
        self.rating.replace(rating);
        self
    }

    /// Session type used to avoid duplicates.
    pub fn session(&mut self, session: SessionType) -> &mut Self {
        self.session.replace(session.to_repr());
        match session {
            SessionType::Ip => {
                self.id.take();
            }
            SessionType::Id(id) => {
                self.id.replace(id);
            }
        }
        self
    }
}

#[derive(Debug, Clone)]
pub enum SessionType {
    /// Session identified based on the User's IP Address.
    /// This is useful when you do not have a unique user identifier.
    Ip,
    /// Session identified based on a unique user identifier passed in the `id` parameter.
    /// This can be, for example, a specific person's ID on Discord.
    Id(String),
}

impl SessionType {
    fn to_repr(&self) -> SessionTypeRepr {
        match self {
            SessionType::Ip => SessionTypeRepr::Ip,
            SessionType::Id(_) => SessionTypeRepr::Id,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
enum SessionTypeRepr {
    Ip,
    Id,
}

#[derive(Debug, Clone)]
pub struct GetRandomFromCategoryResponse {
    pub success: bool,
    pub status: u16,
    pub key: Option<String>,
    pub count: u64,
    pub images: Vec<ImageObject>,
}

#[derive(Debug, Deserialize)]
struct ResponseBase {
    success: bool,
    status: u16,
    key: Option<String>,
    count: u64,
}

#[derive(Debug, Deserialize)]
struct SingleResponse {
    #[serde(flatten)]
    base: ResponseBase,
    #[serde(flatten)]
    object: ImageObject,
}

#[derive(Debug, Deserialize)]
struct MultiResponse {
    #[serde(flatten)]
    base: ResponseBase,
    images: Vec<ImageObject>,
}

impl Endpoint for GetRandomFromCategory {
    type Response = GetRandomFromCategoryResponse;

    const PREHANDLE_ERRORS: bool = true;

    fn build_request(
        &self,
        ctx: super::BuildRequestContext<'_>,
    ) -> Result<reqwest::Request, crate::client::error::Error> {
        let mut url = ctx
            .base_url
            .join(format!("images/{}", self.category).as_str())?;
        url.set_query(serde_qs::to_string(self).ok().as_deref());
        Ok(reqwest::Request::new(Method::GET, url))
    }

    async fn handle_response(
        &self,
        response: reqwest::Response,
    ) -> Result<Self::Response, crate::client::error::Error> {
        if self.count.is_some_and(|count| count.get() > 1) {
            let res: MultiResponse = handle_response_as_json(response).await?;
            Ok(GetRandomFromCategoryResponse {
                success: res.base.success,
                status: res.base.status,
                key: res.base.key,
                count: res.base.count,
                images: res.images,
            })
        } else {
            let res: SingleResponse = handle_response_as_json(response).await?;
            Ok(GetRandomFromCategoryResponse {
                success: res.base.success,
                status: res.base.status,
                key: res.base.key,
                count: res.base.count,
                images: vec![res.object],
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_string_empty() {
        let endpoint = GetRandomFromCategory::new("catgirl");

        assert_eq!("", serde_qs::to_string(&endpoint).unwrap().as_str());
    }

    #[test]
    fn test_query_string() {
        let mut endpoint = GetRandomFromCategory::new("catgirl");

        endpoint
            .count(NonZeroU8::new(10).unwrap())
            .unwrap()
            .additional_tag("aaaa")
            .unwrap()
            .additional_tag("asdd")
            .unwrap()
            .blacklisted_tag("neko")
            .unwrap()
            .blacklisted_tag("opd-a")
            .unwrap();

        assert_eq!(
            "count=10&additionalTags=aaaa,asdd&blacklistedTags=neko,opd-a",
            serde_qs::to_string(&endpoint).unwrap().as_str()
        );
    }

    #[test]
    fn test_query_string_with_session() {
        let mut endpoint = GetRandomFromCategory::new("catgirl");

        {
            endpoint
                .count(NonZeroU8::new(10).unwrap())
                .unwrap()
                .additional_tag("aaaa")
                .unwrap()
                .additional_tag("asdd")
                .unwrap()
                .blacklisted_tag("neko")
                .unwrap()
                .blacklisted_tag("opd-a")
                .unwrap()
                .session(SessionType::Ip);

            assert_eq!(
                "count=10&additionalTags=aaaa,asdd&blacklistedTags=neko,opd-a&session=ip",
                serde_qs::to_string(&endpoint).unwrap().as_str()
            );
        }
        {
            endpoint.session(SessionType::Id("oooo".into()));
            assert_eq!(
                "count=10&additionalTags=aaaa,asdd&blacklistedTags=neko,opd-a&session=id&id=oooo",
                serde_qs::to_string(&endpoint).unwrap().as_str()
            );
        }
    }
}
