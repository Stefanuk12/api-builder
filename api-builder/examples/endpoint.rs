// Dependencies
use api_builder::Endpoint;

#[derive(serde::Deserialize)]
struct Response {
    _success: bool,
}

#[derive(serde::Serialize)]
struct Payload {
    id: String,
    test: String,
}
#[api_builder_derive::api_endpoint(method = GET, path = "\"ab\"", self_as_body = "application/json")]
impl Endpoint for Payload {}

fn main() {}
