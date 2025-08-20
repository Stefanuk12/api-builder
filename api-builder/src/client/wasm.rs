use bytes::Bytes;
use gloo_net::http::{Headers, RequestBuilder};
use http::{Method, Request, Response, header::HeaderValue};
use js_sys::Uint8Array;
use web_sys::RequestCredentials;

use crate::{APIError, AsyncClient, RestClient};

pub trait WasmClient: RestClient {}
impl<C> AsyncClient for C
where
    C: WasmClient + Sync,
{
    async fn rest_async(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Bytes>, APIError<Self::Error>> {
        let headers = Headers::new();
        request.headers().iter().for_each(|(key, value)| {
            if let Ok(value) = value.to_str() {
                headers.append(key.as_str(), value);
            }
        });

        let response = RequestBuilder::new(request.uri().to_string().as_str())
            .credentials(RequestCredentials::Include)
            .method(request.method().clone())
            .headers(headers);

        let response = match request.method() {
            &Method::GET | &Method::HEAD => response.send().await?,
            _ => {
                response
                    .body(Uint8Array::from(request.body().as_slice()))?
                    .send()
                    .await?
            }
        };

        let mut res = Response::builder().status(response.status());

        for (key, value) in response.headers().entries() {
            let Ok(value) = HeaderValue::from_str(value.as_str()) else {
                continue;
            };

            res = res.header(key.as_str(), value);
        }

        Ok(res.body(response.binary().await?.into())?)
    }
}
