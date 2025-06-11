use api_builder::error::APIClientError;

/// Any client errors.
#[derive(Debug, thiserror::Error)]
pub enum APIError {}
impl APIClientError for APIError { }

/// The main client.
#[derive(api_builder::ReqwestAsyncClient)]
pub struct Client {
    /// Inner reqwest client.
    async_client: reqwest::Client,
}
#[api_builder::api_rest_client(error = APIError, base = "\"https://example.com/v1/\"")]
impl api_builder::client::RestClient for Client {}

fn main() {}
