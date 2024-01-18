// Dependencies
use api_builder::{Endpoint, Query as _};

/// Any client errors.
#[derive(Debug, thiserror::Error)]
pub enum APIError {}

/// The main client.
#[derive(api_builder::ReqwestClient)]
pub struct Client {
    /// Inner reqwest client.
    client: reqwest::blocking::Client,
}
#[api_builder::api_rest_client(error = APIError, base = "\"https://example.com/v1/\"")]
impl api_builder::client::RestClient for Client {}
impl Default for Client {
    fn default() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}

/// The expected response for the resource below.
#[derive(serde::Deserialize)]
struct Response {
    _success: bool,
}

/// The main resource.
#[derive(serde::Serialize)]
struct Payload {
    id: String,
    test: String,
}

// Automatically implements `Endpoint` for `Payload`.
#[api_builder_derive::api_endpoint(method = GET, path = "\"ab\"", self_as_body = "application/json")]
impl Endpoint for Payload {}

// Add additional methods to the resource.
impl Payload {
    /// A wrapper around the `query` method that can be modified to add custom logic, should be an in-place replacement for `query`.
    ///
    /// You could also create a combinator.
    pub fn final_query<C: api_builder::Client>(
        &self,
        client: &C,
    ) -> Result<Response, api_builder::error::APIError<C::Error>> {
        api_builder::query::Query::<Response, C>::finalise(
            self,
            api_builder::query::Query::<Response, C>::send(
                self,
                client,
                api_builder::query::Query::<Response, C>::request(self, client)?,
            )?,
        )
    }
}

fn main() {
    let client = Client::default();

    let payload = Payload {
        id: "test".to_string(),
        test: "test".to_string(),
    };

    let _response = payload.final_query(&client).unwrap();
    let _response: Response = payload.query(&client).unwrap();
}
