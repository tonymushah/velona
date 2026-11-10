use reqwest::Method;
use serde::Deserialize;

use crate::client::endpoint::{Endpoint, handle_response_as_json};

#[derive(Debug, Clone, Default)]
pub struct GetTags;

#[derive(Debug, Deserialize)]
pub struct GetTagsResponse {
    pub success: bool,
    pub status: u16,
    pub tags: Vec<String>,
    pub anime: Vec<String>,
    pub characters: Vec<String>,
}

impl Endpoint for GetTags {
    type Response = GetTagsResponse;

    const PREHANDLE_ERRORS: bool = true;

    fn build_request(
        &self,
        ctx: super::BuildRequestContext<'_>,
    ) -> Result<reqwest::Request, crate::client::error::Error> {
        Ok(reqwest::Request::new(
            Method::GET,
            ctx.base_url.join("tags")?,
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
    async fn test_tags_handling() {
        let (server, client) = create_wiremock_client().await;

        let response = json!({
          "success": true,
          "status": 200,
          "tags": [
              "catgirls",
              "uniform",
              "after-rain"
          ],
          "anime": [
              "Blue Archive",
              "Genshin Impact",
              "Zenless Zone Zero"
          ],
          "characters": [
              "Yuka",
              "Asuna",
              "Lumine",
              "Odette",
              "Alice",
              "Amby"
          ]
        });

        Mock::given(method("GET"))
            .and(path("/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&server)
            .await;

        let res = client.send_endpoint(GetTags).await.unwrap();

        assert!(res.success);

        assert_eq!(res.tags.len(), 3);

        assert_eq!(res.anime.len(), 3);

        assert_eq!(res.characters.len(), 6);
    }

    #[tokio::test]
    async fn test_tags_error_parsing() {
        let (server, client) = create_wiremock_client().await;

        let response = json!({
          "success": false,
          "status": 500,
          "message": "Internal Error"
        });

        Mock::given(method("GET"))
            .and(path("/tags"))
            .respond_with(ResponseTemplate::new(500).set_body_json(response))
            .mount(&server)
            .await;

        let res = client.send_endpoint(GetTags).await.unwrap_err();

        assert_eq!(
            match res {
                crate::client::error::Error::NekosiaRequest(err) => err.message,
                err => unreachable!("{:#?}", err),
            },
            "Internal Error"
        )
    }
}
