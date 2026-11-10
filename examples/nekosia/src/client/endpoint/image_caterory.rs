use std::num::NonZeroU8;

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
    use serde_json::json;
    use wiremock::{
        Mock, ResponseTemplate,
        matchers::{method, path, query_param},
    };

    use crate::client::for_tests::create_wiremock_client;

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

    #[tokio::test]
    async fn test_single_parsing() {
        let (server, client) = create_wiremock_client().await;

        let response = json!({
            "success": true,
            "status": 200,
            "count": 1608,
            "id": "66e0c27d8358c411f61a94fe",
            "colors": {
                "main": "#ECC3BC",
                "palette": [
                    "#E7CDDF",
                    "#EECCCC",
                    "#BF6E7B",
                    "#D79696",
                    "#71484F",
                    "#654F5D",
                    "#F7EFDB",
                    "#A07A8F",
                    "#EF7E99",
                    "#FDF2F0",
                    "#E983AE",
                    "#F6ACB0",
                    "#F3F5F1",
                    "#99B8E5"
                ]
            },
            "image": {
                "original": {
                    "url": "https://cdn.nekosia.cat/images/catgirl/66e0c27d8358c411f61a94fe.jpg",
                    "extension": "jpeg"
                },
                "compressed": {
                    "url": "https://cdn.nekosia.cat/images/catgirl/66e0c27d8358c411f61a94fe-compressed.jpg",
                    "extension": "jpeg"
                }
            },
            "metadata": {
                "original": {
                    "width": 1290,
                    "height": 1821,
                    "size": 2117089,
                    "extension": "jpeg",
                    "colorProfile": null
                },
                "compressed": {
                    "width": 1280,
                    "height": 1807,
                    "size": 508294,
                    "extension": "jpeg",
                    "colorProfile": null
                }
            },
            "category": "catgirl",
            "tags": [
                "catgirl",
                "cat-ears",
                "animal-ears",
                "tail",
                "cute",
                "w-sitting",
                "blue-eyes",
                "blushing",
                "pink-hair",
                "rawr",
                "skirt",
                "twintails"
            ],
            "rating": "safe",
            "anime": {
                "title": null,
                "character": null
            },
            "source": {
                "url": "https://www.pixiv.net/en/artworks/105116761",
                "direct": "https://i.pximg.net/img-original/img/2023/02/05/20/53/38/105116761_p0.jpg"
            },
            "attribution": {
                "artist": {
                    "username": "めいび",
                    "profile": "https://www.pixiv.net/en/users/89796887"
                },
                "copyright": "Copyright 2023 © by めいび. All Rights Reserved."
            }
        });

        Mock::given(method("GET"))
            .and(path("/images/cat-girl"))
            .and(query_param("additionalTags", "skirt,twintails"))
            .and(query_param("session", "ip"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&server)
            .await;

        let res = client
            .send_endpoint({
                let mut endpoint = GetRandomFromCategory::new("cat-girl");
                endpoint
                    .additional_tag("skirt")
                    .unwrap()
                    .additional_tag("twintails")
                    .unwrap()
                    .session(SessionType::Ip);
                endpoint
            })
            .await
            .unwrap();

        assert!(res.success);

        assert_eq!(res.images.len(), 1);

        assert_eq!(res.images[0].tags.len(), 12);
    }

    #[tokio::test]
    async fn test_multiple_parsing() {
        let (server, client) = create_wiremock_client().await;

        let response = json!({
          "success": true,
          "status": 200,
          "count": 1608,
          "images": [
            {
              "id": "67f1afb50d94aa59fbc9811d",
              "colors": {
                "main": "#5373BD",
                "palette": [
                  "#F2BAB8",
                  "#1266B1",
                  "#E7E8FD",
                  "#B2769A",
                  "#A18EAE",
                  "#76BEEC",
                  "#3F2F4F",
                  "#A65D69",
                  "#612E39",
                  "#DE7B90",
                  "#E2FCE3",
                  "#2E5CE4",
                  "#50E8FF",
                  "#C8446C"
                ]
              },
              "image": {
                "original": {
                  "url": "https://cdn.nekosia.cat/images/catgirl/67f1afb50d94aa59fbc9811d.png",
                  "extension": "png"
                },
                "compressed": {
                  "url": "https://cdn.nekosia.cat/images/catgirl/67f1afb50d94aa59fbc9811d-compressed.jpg",
                  "extension": "jpeg"
                }
              },
              "metadata": {
                "original": {
                  "width": 1254,
                  "height": 1771,
                  "size": 1739522,
                  "extension": "png",
                  "colorProfile": null
                },
                "compressed": {
                  "width": 1280,
                  "height": 1808,
                  "size": 446282,
                  "extension": "jpeg",
                  "colorProfile": null
                }
              },
              "category": "catgirl",
              "tags": [
                "catgirl",
                "cat-ears",
                "animal-ears",
                "tail",
                "cute",
                "young-girl",
                "skirt",
                "socks",
                "tail-from-under-skirt",
                "uniform"
              ],
              "rating": "safe",
              "anime": {
                "title": null,
                "character": null
              },
              "source": {
                "url": "https://www.pixiv.net/en/artworks/128820223",
                "direct": "https://i.pximg.net/img-original/img/2025/04/01/00/00/29/128820223_p0.png"
              },
              "attribution": {
                "artist": {
                  "username": "ぴろ瀬",
                  "profile": "https://www.pixiv.net/en/users/19731971"
                },
                "copyright": "Copyright 2025 © by ぴろ瀬. All Rights Reserved."
              }
            },
            {
              "id": "6859b280acca2a268bbf97ed",
              "colors": {
                "main": "#E0C5B9",
                "palette": [
                  "#D9CED8",
                  "#D8B8AE",
                  "#A75C45",
                  "#542E29",
                  "#BBDDD4",
                  "#BA8B7A",
                  "#E66F73",
                  "#F1DCD5",
                  "#EFECDF",
                  "#473443",
                  "#F9F0EC",
                  "#77617D",
                  "#EC9B97",
                  "#DD71A2"
                ]
              },
              "image": {
                "original": {
                  "url": "https://cdn.nekosia.cat/images/catgirl/6859b280acca2a268bbf97ed.jpg",
                  "extension": "jpeg"
                },
                "compressed": {
                  "url": "https://cdn.nekosia.cat/images/catgirl/6859b280acca2a268bbf97ed-compressed.jpg",
                  "extension": "jpeg"
                }
              },
              "metadata": {
                "original": {
                  "width": 1191,
                  "height": 1860,
                  "size": 1871927,
                  "extension": "jpeg",
                  "colorProfile": "Display P3"
                },
                "compressed": {
                  "width": 1280,
                  "height": 1999,
                  "size": 584995,
                  "extension": "jpeg",
                  "colorProfile": null
                }
              },
              "category": "catgirl",
              "tags": [
                "catgirl",
                "cat-ears",
                "animal-ears",
                "cute",
                "blushing",
                "donut",
                "white-hair",
                "yellow-eyes"
              ],
              "rating": "safe",
              "anime": {
                "title": null,
                "character": null
              },
              "source": {
                "url": "https://www.pixiv.net/en/artworks/92774035",
                "direct": "https://i.pximg.net/img-original/img/2021/09/15/23/45/53/92774035_p0.jpg"
              },
              "attribution": {
                "artist": {
                  "username": "ツユハ",
                  "profile": "https://www.pixiv.net/en/users/29694453"
                },
                "copyright": "Copyright 2021 © by ツユハ. All Rights Reserved."
              }
            }
          ]
        });

        Mock::given(method("GET"))
            .and(path("/images/cat-girl"))
            .and(query_param("additionalTags", "skirt,twintails"))
            .and(query_param("count", "2"))
            .and(query_param("session", "id"))
            .and(query_param("id", "2345"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&server)
            .await;

        let res = client
            .send_endpoint({
                let mut endpoint = GetRandomFromCategory::new("cat-girl");
                endpoint
                    .additional_tag("skirt")
                    .unwrap()
                    .additional_tag("twintails")
                    .unwrap()
                    .count(NonZeroU8::new(2).unwrap())
                    .unwrap()
                    .session(SessionType::Id("2345".into()));
                endpoint
            })
            .await
            .unwrap();

        assert!(res.success);

        assert_eq!(res.images.len(), 2);

        assert_eq!(res.images[0].tags.len(), 10);

        assert_eq!(res.images[0].category, "catgirl");

        assert_eq!(res.images[1].tags.len(), 8);

        assert_eq!(res.images[1].category, "catgirl");
    }

    #[tokio::test]
    async fn test_image_category_error_parsing() {
        let (server, client) = create_wiremock_client().await;

        let response = json!({
          "success": false,
          "status": 500,
          "message": "Internal Error"
        });

        Mock::given(method("GET"))
            .and(path("/images/cat-girl"))
            .and(query_param("additionalTags", "skirt,twintails"))
            .and(query_param("count", "2"))
            .and(query_param("session", "id"))
            .and(query_param("id", "2345"))
            .respond_with(ResponseTemplate::new(500).set_body_json(response))
            .mount(&server)
            .await;

        let res = client
            .send_endpoint({
                let mut endpoint = GetRandomFromCategory::new("cat-girl");
                endpoint
                    .additional_tag("skirt")
                    .unwrap()
                    .additional_tag("twintails")
                    .unwrap()
                    .count(NonZeroU8::new(2).unwrap())
                    .unwrap()
                    .session(SessionType::Id("2345".into()));
                endpoint
            })
            .await
            .unwrap_err();

        assert_eq!(
            match res {
                crate::client::error::Error::NekosiaRequest(err) => err.message,
                _ => unreachable!(),
            },
            "Internal Error"
        )
    }

    #[tokio::test]
    async fn test_more_parsing_1() {
        let (server, client) = create_wiremock_client().await;

        let response = json!({
          "success": true,
          "status": 200,
          "key": "user:1358c:viewed:hololive",
          "count": 47,
          "id": "6823e3559ca50413d4de154e",
          "colors": {
            "main": "#BD9A8B",
            "palette": [
              "#3A3347",
              "#BC9183",
              "#C7CCDF",
              "#5F607B",
              "#7EC1AF",
              "#F5E2CA",
              "#0086E2",
              "#453637",
              "#665252",
              "#7F8BAB",
              "#8DB8F9",
              "#D5ACC4",
              "#222992",
              "#DA3A43"
            ]
          },
          "image": {
            "original": {
              "url": "https://cdn.nekosia.cat/images/vtuber/6823e3559ca50413d4de154e.png",
              "extension": "png"
            },
            "compressed": {
              "url": "https://cdn.nekosia.cat/images/vtuber/6823e3559ca50413d4de154e-compressed.jpg",
              "extension": "jpeg"
            }
          },
          "metadata": {
            "original": {
              "width": 2000,
              "height": 2870,
              "size": 5551313,
              "extension": "png",
              "colorProfile": null
            },
            "compressed": {
              "width": 1280,
              "height": 1837,
              "size": 547272,
              "extension": "jpeg",
              "colorProfile": null
            }
          },
          "category": "vtuber",
          "tags": [
            "vtuber",
            "schoolgirl",
            "catgirl",
            "cat-ears",
            "animal-ears",
            "tail",
            "cute",
            "young-girl",
            "thigh-high-socks",
            "white-thigh-high-socks",
            "butterfly",
            "hololive",
            "red-eyes",
            "school-uniform",
            "short-hair",
            "socks",
            "streaked-hair",
            "uniform",
            "uruha-rushia",
            "valentine"
          ],
          "rating": "safe",
          "anime": { "title": null, "character": "Uruha Rushia" },
          "source": {
            "url": "https://www.pixiv.net/en/artworks/105347321",
            "direct": "https://i.pximg.net/img-original/img/2023/02/13/21/09/47/105347321_p0.png"
          },
          "attribution": {
            "artist": {
              "username": "ネコザメ🦈",
              "profile": "https://www.pixiv.net/en/users/51853916"
            },
            "copyright": "Copyright 2023 © by ネコザメ🦈. All Rights Reserved."
          }
        });

        Mock::given(method("GET"))
            .and(path("/images/hololive"))
            .and(query_param("session", "id"))
            .and(query_param("id", "1358c"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&server)
            .await;

        let res = client
            .send_endpoint({
                let mut endpoint = GetRandomFromCategory::new("hololive");
                endpoint.session(SessionType::Id("1358c".into()));
                endpoint
            })
            .await
            .unwrap();

        assert!(res.success);

        assert_eq!(res.images.len(), 1);

        assert_eq!(res.images[0].tags.len(), 20);

        assert_eq!(res.images[0].tags[0], "vtuber");

        assert_eq!(res.images[0].category, "hololive");
    }
}
