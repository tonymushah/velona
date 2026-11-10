pub mod endpoint;
pub mod error;
pub mod objects;

use std::{num::NonZeroU32, sync::Arc, time::Duration};

use governor::{DefaultDirectRateLimiter, Jitter, Quota};
use reqwest::{Client, Request, Response};
use tokio::sync::RwLock;
use url::Url;
use velona::reactive::owner::{expect_context, provide_context};

use crate::client::endpoint::{BuildRequestContext, Endpoint};

pub const NEKOSIA_API_URL: &str = "https://api.nekosia.cat/api/v1/";

pub const RATE_LIMIT_QUOTA: u32 = 300;

pub const RATE_LIMIT_PERIOD_SECONDS: u64 = 300 * 60;

#[derive(Debug)]
pub struct NekosiaClient {
    reqwest_client: Client,
    base_url: Url,
    rate_limiter: DefaultDirectRateLimiter,
}

impl NekosiaClient {
    pub fn new(reqwest_client: Client) -> NekosiaClient {
        Self::new_with_base_url(reqwest_client, Url::parse(NEKOSIA_API_URL).unwrap())
    }

    pub fn new_with_base_url(reqwest_client: Client, base_url: Url) -> NekosiaClient {
        NekosiaClient {
            reqwest_client,
            base_url,
            rate_limiter: DefaultDirectRateLimiter::direct(
                Quota::with_period(Duration::from_secs(RATE_LIMIT_PERIOD_SECONDS))
                    .map(|e| e.allow_burst(NonZeroU32::new(RATE_LIMIT_QUOTA).unwrap()))
                    .unwrap_or(Quota::per_minute(NonZeroU32::new(10).unwrap())),
            ),
        }
    }

    pub fn rate_limiter(mut self, rate_limiter: DefaultDirectRateLimiter) -> NekosiaClient {
        self.rate_limiter = rate_limiter;
        self
    }

    pub fn use_reqwest_client(&self) -> &Client {
        &self.reqwest_client
    }
}

impl NekosiaClient {
    pub async fn wait_rate_limiter(&self) {
        self.rate_limiter
            .until_ready_with_jitter(default_jitter())
            .await;
    }

    pub async fn send_request(&self, request: Request) -> Result<Response, error::Error> {
        let response = self.send_request_without_handling_error(request).await?;
        if response.status().is_success() {
            Ok(response)
        } else if response.status() == 429 {
            Err(error::Error::TooManyRequests)
        } else {
            let err = response.json::<error::RequestError>().await?;
            Err(Box::new(err).into())
        }
    }

    async fn send_request_without_handling_error(
        &self,
        request: Request,
    ) -> Result<Response, error::Error> {
        self.wait_rate_limiter().await;
        let response = self.reqwest_client.execute(request).await?;
        Ok(response)
    }
    pub async fn send_endpoint<E>(&self, endpoint: E) -> Result<E::Response, error::Error>
    where
        E: Endpoint + 'static,
    {
        let request = endpoint.build_request(BuildRequestContext {
            base_url: &self.base_url,
        })?;
        let response = if E::PREHANDLE_ERRORS {
            self.send_request(request).await?
        } else {
            self.send_request_without_handling_error(request).await?
        };
        endpoint.handle_response(response).await
    }
}

#[cfg(test)]
pub mod for_tests {
    use wiremock::MockServer;

    use super::*;
    pub async fn create_wiremock_client() -> (MockServer, NekosiaClient) {
        let mock_server = MockServer::start().await;

        let client = NekosiaClient::new_with_base_url(
            Client::new(),
            Url::parse(mock_server.uri().as_str()).unwrap(),
        );

        (mock_server, client)
    }
}

pub fn provide_client(client: NekosiaClient) {
    provide_context(Arc::new(RwLock::new(client)));
}

pub fn expect_client() -> Arc<RwLock<NekosiaClient>> {
    expect_context()
}

fn default_jitter() -> Jitter {
    Jitter::new(Duration::from_millis(500), Duration::from_secs(2))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_val_send<T: Send>(_val: &T) {}

    struct DummyEndPoint {}

    struct DummyResponse;

    impl Endpoint for DummyEndPoint {
        type Response = DummyResponse;

        const PREHANDLE_ERRORS: bool = true;

        fn build_request(
            &self,
            _ctx: BuildRequestContext<'_>,
        ) -> Result<Request, super::error::Error> {
            todo!()
        }

        async fn handle_response(
            &self,
            _response: Response,
        ) -> Result<Self::Response, super::error::Error> {
            todo!()
        }
    }

    #[test]
    fn test_send_sync() {
        let client = NekosiaClient::new(Client::new());
        let fut = client.send_endpoint(DummyEndPoint {});
        is_val_send(&fut);
    }
}
