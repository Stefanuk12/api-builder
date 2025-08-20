use api_builder::{Endpoint, api_endpoint};

#[derive(serde::Deserialize)]
struct Response {
    _success: bool,
}

#[derive(serde::Serialize)]
struct _Payload {
    id: String,
    test: String,
}
#[api_endpoint(method = GET, path = "\"ab\"", self_as_body = "application/json")]
impl Endpoint for _Payload {}

fn main() {}
