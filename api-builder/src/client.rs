// Dependencies
use crate::error::APIError;
use bytes::Bytes;
use http::Response;

/// A trait representing a client which can communicate with an instance via REST.
pub trait RestClient {
    /// The errors which may occur for this client.
    type Error: crate::error::APIClientError;

    /// Get the URL for the endpoint for the client.
    ///
    /// This method adds the hostname for the client's target instance.
    fn rest_endpoint(&self, path: &str) -> Result<url::Url, APIError<Self::Error>>;
}

/// A trait represnting a client which includes a reqwest client.
#[cfg(feature = "reqwest")]
pub trait ReqwestAsyncClient: RestClient {
    /// Get the reqwest client.
    fn client(&self) -> &reqwest::Client;
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

/// A trait representing a client.
pub trait Client: RestClient {
    /// Send a REST query.
    fn rest(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> Result<Response<Bytes>, APIError<Self::Error>>;
}

#[cfg(feature = "reqwest_blocking")]
#[cfg(not(target_family = "wasm"))]
impl<C> Client for C
where
    C: RestClient + ReqwestClient,
{
    fn rest(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> Result<Response<Bytes>, APIError<Self::Error>> {
        // Send the request
        let response = self.client().execute(request.try_into()?)?;

        // Construct the response builder
        let mut http_response = http::Response::builder()
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

/// A trait representing an asynchronous client.
pub trait AsyncClient: RestClient {
    #[cfg(not(target_arch = "wasm32"))]
    /// Send a REST query asynchronously.
    fn rest_async(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> impl std::future::Future<Output = Result<Response<Bytes>, APIError<Self::Error>>> + Send;

    #[cfg(target_arch = "wasm32")]
    /// Send a REST query asynchronously.
    fn rest_async(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> impl std::future::Future<Output = Result<Response<Bytes>, APIError<Self::Error>>>;
}

#[cfg(feature = "reqwest")]
impl<C> AsyncClient for C
where
    C: ReqwestAsyncClient + Sync,
{
    async fn rest_async(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> Result<Response<Bytes>, APIError<Self::Error>> {
        // Send the request
        let response = self.client().execute(request.try_into()?).await?;

        // Construct the response builder
        #[allow(unused_mut)]
        let mut http_response = http::Response::builder().status(response.status());

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

#[cfg(target_family = "wasm")]
#[cfg(not(feature = "reqwest"))]
pub trait WasmClient: RestClient {}
#[cfg(target_family = "wasm")]
#[cfg(not(feature = "reqwest"))]
impl<C> AsyncClient for C
where
    C: WasmClient + Sync,
{
    async fn rest_async(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> Result<Response<Bytes>, APIError<Self::Error>> {
        use gloo_net::http::{Headers, RequestBuilder};
        pub use http::{header::HeaderValue, Response};
        use web_sys::RequestCredentials;

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
            &http::Method::GET | &http::Method::HEAD => response.send().await?,
            _ => {
                response
                    .body(js_sys::Uint8Array::from(request.body().as_slice()))?
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
