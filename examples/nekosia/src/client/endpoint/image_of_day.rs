use reqwest::Method;
use serde::{Deserialize, Deserializer};
use time::{Date, format_description::StaticFormatDescription, macros::format_description};

use crate::client::{
    endpoint::{Endpoint, handle_response_as_json},
    objects::image::ImageObject,
};

#[derive(Debug, Clone, Default)]
pub struct ImageOfTheDay;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageOfTheDayResponse {
    pub success: bool,
    pub status: u16,
    #[serde(deserialize_with = "deserialize_date")]
    pub date: Date,
    #[serde(flatten)]
    pub object: ImageObject,
}

const IOFTD_FORMAT: StaticFormatDescription = format_description!("[year]-[month]-[day]");

fn deserialize_date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: Deserializer<'de>,
{
    Date::parse(&String::deserialize(deserializer)?, IOFTD_FORMAT).map_err(serde::de::Error::custom)
}

impl Endpoint for ImageOfTheDay {
    type Response = ImageOfTheDayResponse;

    const PREHANDLE_ERRORS: bool = true;

    fn build_request(
        &self,
        ctx: super::BuildRequestContext<'_>,
    ) -> Result<reqwest::Request, crate::client::error::Error> {
        Ok(reqwest::Request::new(
            Method::GET,
            ctx.base_url.join("image-of-the-day")?,
        ))
    }

    fn handle_response(
        &self,
        response: reqwest::Response,
    ) -> impl Future<Output = Result<Self::Response, crate::client::error::Error>> + Send {
        handle_response_as_json(response)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::{
        Mock, ResponseTemplate,
        matchers::{method, path},
    };

    use crate::client::for_tests::create_wiremock_client;

    use super::*;

    #[tokio::test]
    async fn test_image_of_day_handling() {
        let (server, client) = create_wiremock_client().await;

        let response = json!({
          "success": true,
          "status": 200,
          "date": "2026-10-23",
          "id": "66aa41f6c8193e5c010ffbf5",
          "colors": {
            "main": "#F2DFCC",
            "palette": [
              "#4D3E4D",
              "#442E2D",
              "#F6FCF8",
              "#FAE8D7",
              "#D5D0DE",
              "#A88273",
              "#F8FBF6",
              "#F8C68C",
              "#FBD3A5",
              "#E0D2D2",
              "#CEAB9B",
              "#C95846",
              "#5A5953",
              "#A24940"
            ]
          },
          "image": {
            "original": {
              "url": "https://cdn.nekosia.cat/images/catgirl/66aa41f6c8193e5c010ffbf5.jpg",
              "extension": "jpeg"
            },
            "compressed": {
              "url": "https://cdn.nekosia.cat/images/catgirl/66aa41f6c8193e5c010ffbf5-compressed.jpg",
              "extension": "jpeg"
            }
          },
          "metadata": {
            "original": {
              "width": 2480,
              "height": 3508,
              "size": 1239952,
              "extension": "jpeg",
              "colorProfile": null
            },
            "compressed": {
              "width": 1280,
              "height": 1811,
              "size": 341182,
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
            "young-girl",
            "black-hair",
            "long-hair",
            "purple-eyes",
            "showgirl"
          ],
          "rating": "safe",
          "anime": {
            "title": null,
            "character": null
          },
          "source": {
            "url": "https://www.pixiv.net/en/artworks/81566707",
            "direct": "https://i.pximg.net/img-original/img/2020/05/15/00/04/39/81566707_p0.jpg"
          },
          "attribution": {
            "artist": {
              "username": "fufumi",
              "profile": "https://www.pixiv.net/en/users/11008221"
            },
            "copyright": "Copyright 2020 © by fufumi. All Rights Reserved."
          }
        });

        Mock::given(method("GET"))
            .and(path("/image-of-the-day"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&server)
            .await;

        let res = client.send_endpoint(ImageOfTheDay).await.unwrap();

        assert!(res.success);

        assert_eq!(res.object.tags.len(), 9);

        assert_eq!(res.object.category, "catgirl");
    }

    #[tokio::test]
    async fn test_iod_error_parsing() {
        let (server, client) = create_wiremock_client().await;

        let response = json!({
          "success": false,
          "status": 500,
          "message": "Internal Error"
        });

        Mock::given(method("GET"))
            .and(path("/image-of-the-day"))
            .respond_with(ResponseTemplate::new(500).set_body_json(response))
            .mount(&server)
            .await;

        let res = client.send_endpoint(ImageOfTheDay).await.unwrap_err();

        assert_eq!(
            match res {
                crate::client::error::Error::NekosiaRequest(err) => err.message,
                err => unreachable!("{:#?}", err),
            },
            "Internal Error"
        )
    }
}
