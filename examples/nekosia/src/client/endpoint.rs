pub mod image_caterory;

use reqwest::{Request, Response};
use serde::de::DeserializeOwned;
use url::Url;

#[derive(Debug)]
pub struct BuildRequestContext<'a> {
    pub base_url: &'a Url,
}

pub trait Endpoint {
    type Response: 'static;

    const PREHANDLE_ERRORS: bool;

    fn build_request(&self, ctx: BuildRequestContext<'_>) -> Request;

    fn handle_response(
        &self,
        response: Response,
    ) -> impl Future<Output = Result<Self::Response, super::error::Error>> + Send;
}

pub async fn handle_response_as_json<R>(respone: Response) -> Result<R, super::error::Error>
where
    R: DeserializeOwned,
{
    Ok(respone.json::<R>().await?)
}
