use std::{future::Pending, num::NonZeroU8};

use arrayvec::ArrayVec;
use serde::Serialize;

use crate::client::endpoint::Endpoint;

type Tags20Array = ArrayVec<String, 20>;

/// This endpoint allows you to fetch random images from a selected category.
///
/// Check [https://nekosia.cat/documentation?page=endpoints#get-image-category] for more details
#[derive(Debug, Serialize)]
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
}

pub struct GetRandomFromCategoryResponse {}

impl Endpoint for GetRandomFromCategory {
    type Response = GetRandomFromCategoryResponse;

    const PREHANDLE_ERRORS: bool = true;

    fn build_request(&self, ctx: super::BuildRequestContext<'_>) -> reqwest::Request {
        todo!()
    }

    fn handle_response(
        &self,
        response: reqwest::Response,
    ) -> impl Future<Output = Result<Self::Response, crate::client::error::Error>> + Send {
        todo!() as Pending<_>
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
            "count=10&additional_tags=aaaa,asdd&blacklisted_tags=neko,opd-a",
            serde_qs::to_string(&endpoint).unwrap().as_str()
        );
    }
}
