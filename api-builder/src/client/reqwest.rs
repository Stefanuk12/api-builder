use bytes::Bytes;
use http::{Request, Response};

use crate::RestClient;
use crate::{APIError, AsyncClient};

/// A trait represnting a client which includes a reqwest client.
#[cfg(feature = "reqwest")]
pub trait ReqwestAsyncClient: RestClient {
    /// Get the reqwest client.
    fn client(&self) -> &reqwest::Client;
}

#[cfg(feature = "reqwest")]
impl<C> AsyncClient for C
where
    C: ReqwestAsyncClient + Sync,
{
    async fn rest_async(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Bytes>, APIError<Self::Error>> {
        // Send the request
        let response = self.client().execute(request.try_into()?).await?;

        // Construct the response builder
        #[allow(unused_mut)]
        let mut http_response = Response::builder().status(response.status());

        #[cfg(not(target_family = "wasm"))]
        let mut http_response = http_response.version(response.version());

        // Add headers
        let headers = http_response.headers_mut().unwrap();
        for (key, value) in response.headers() {
            headers.append(key, value.clone());
        }

        // Add the body and return the response
        Ok(http_response.body(response.bytes().await?)?)
    }
}

/// A trait representing a client which includes a blocking reqwest client.
///
/// Does not work on wasm.
#[cfg(feature = "reqwest_blocking")]
#[cfg(not(target_family = "wasm"))]
pub trait ReqwestClient: RestClient {
    /// Get the blocking reqwest client.
    fn client(&self) -> &reqwest::blocking::Client;
}

#[cfg(feature = "reqwest_blocking")]
#[cfg(not(target_family = "wasm"))]
impl<C> Client for C
where
    C: RestClient + ReqwestClient,
{
    fn rest(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, APIError<Self::Error>> {
        // Send the request
        let response = self.client().execute(request.try_into()?)?;

        // Construct the response builder
        let mut http_response = Response::builder()
            .status(response.status())
            .version(response.version());

        // Add headers
        let headers = http_response.headers_mut().unwrap();
        for (key, value) in response.headers() {
            headers.append(key, value.clone());
        }

        // Add the body and return the response
        Ok(http_response.body(response.bytes()?)?)
    }
}
