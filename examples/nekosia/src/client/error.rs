use serde::Deserialize;
use url::Url;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    Reqwest(#[from] reqwest::Error),
    NekosiaRequest(#[from] Box<RequestError>),
    Url(#[from] url::ParseError),
    #[error("Too Many Requests")]
    TooManyRequests,
}

#[derive(Debug, Deserialize, thiserror::Error)]
#[error("{}", .message)]
pub struct RequestError {
    pub status: u16,
    pub message: String,
    pub docs: Option<Url>,
    pub example: Option<String>,
}
