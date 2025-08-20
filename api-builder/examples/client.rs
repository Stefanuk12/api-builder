use api_builder::{api_rest_client, error::APIClientError, ReqwestAsyncClient, RestClient};

/// Any client errors.
#[derive(Debug, thiserror::Error)]
pub enum APIError {}
impl APIClientError for APIError {}

/// The main client.
#[derive(ReqwestAsyncClient)]
pub struct Client {
    /// Inner reqwest client.
    async_client: reqwest::Client,
}
#[api_rest_client(error = APIError, base = "\"https://example.com/v1/\"")]
impl RestClient for Client {}

fn main() {}
