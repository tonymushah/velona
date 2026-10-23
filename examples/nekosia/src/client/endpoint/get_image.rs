use reqwest::Method;
use serde::Deserialize;

use crate::client::{
    endpoint::{Endpoint, handle_response_as_json},
    objects::image_id::ImageObjectWithObjectRating,
};

#[derive(Debug, Clone)]
pub struct GetImageById {
    id: String,
}

impl GetImageById {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetImageByIdResult {
    pub success: bool,
    pub status: u16,
    #[serde(flatten)]
    pub object: ImageObjectWithObjectRating,
}

impl Endpoint for GetImageById {
    type Response = GetImageByIdResult;

    const PREHANDLE_ERRORS: bool = true;

    fn build_request(
        &self,
        ctx: super::BuildRequestContext<'_>,
    ) -> Result<reqwest::Request, crate::client::error::Error> {
        let url = ctx
            .base_url
            .join(format!("getImageById/{}", self.id).as_str())?;
        Ok(reqwest::Request::new(Method::GET, url))
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
    async fn test_get_image_by_id() {
        let (server, client) = create_wiremock_client().await;

        let response = json!({
          "success": true,
          "status": 200,
          "id": "66bc6b7481a59a1cf2c79db5",
          "colors": {
            "main": "#8D6147",
            "palette": [
              "#32374A",
              "#F7D09C",
              "#10111B",
              "#714D3A",
              "#D2966F",
              "#1B120D",
              "#283026",
              "#3D291F",
              "#A97458",
              "#AF6440",
              "#545033",
              "#090F0D",
              "#C34F3D",
              "#B9C0B8"
            ]
          },
          "image": {
            "original": {
              "url": "https://cdn.nekosia.cat/images/maid-uniform/66bc6b7481a59a1cf2c79db5.png",
              "extension": "png"
            },
            "compressed": {
              "url": "https://cdn.nekosia.cat/images/maid-uniform/66bc6b7481a59a1cf2c79db5-compressed.jpg",
              "extension": "jpeg"
            }
          },
          "metadata": {
            "original": {
              "width": 2743,
              "height": 1543,
              "size": 7119459,
              "extension": "png",
              "colorProfile": null
            },
            "compressed": {
              "width": 1280,
              "height": 720,
              "size": 270400,
              "extension": "jpeg",
              "colorProfile": null
            }
          },
          "category": "maid-uniform",
          "tags": [
            "maid-uniform",
            "vtuber",
            "catgirl",
            "cat-ears",
            "animal-ears",
            "tail",
            "cute",
            "young-girl",
            "blue-eyes",
            "lying-down",
            "uniform"
          ],
          "rating": {
            "rating": "safe",
            "available": true,
            "nsfw": 0.2,
            "questionable": 12.48,
            "safe": 87.31,
            "suggestive": 0
          },
          "anime": {
            "title": null,
            "character": null
          },
          "source": {
            "url": "https://www.pixiv.net/en/artworks/102097796",
            "direct": "https://i.pximg.net/img-original/img/2022/10/21/00/00/10/102097796_p0.png"
          },
          "attribution": {
            "artist": {
              "username": "MILTEA",
              "profile": "https://www.pixiv.net/en/users/8545086"
            },
            "copyright": "Copyright 2022 © by MILTEA. All Rights Reserved."
          }
        });

        Mock::given(method("GET"))
            .and(path("/getImageById/1234"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&server)
            .await;

        let res = client
            .send_endpoint(GetImageById::new("1234".into()))
            .await
            .unwrap();

        assert!(res.success);

        assert_eq!(res.object.tags.len(), 11);

        assert_eq!(res.object.category, "maid-uniform");
    }

    #[tokio::test]
    async fn test_error_parsing() {
        let (server, client) = create_wiremock_client().await;

        let response = json!({
          "success": false,
          "status": 500,
          "message": "Internal Error"
        });

        Mock::given(method("GET"))
            .and(path("/getImageById/1234"))
            .respond_with(ResponseTemplate::new(500).set_body_json(response))
            .mount(&server)
            .await;

        let res = client
            .send_endpoint(GetImageById::new("1234".into()))
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
}
