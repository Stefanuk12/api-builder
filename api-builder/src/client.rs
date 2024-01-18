// Dependencies
use crate::error::APIError;
use bytes::Bytes;
use http::{request::Builder, Response};

/// A trait representing a client which can communicate with an instance via REST.
pub trait RestClient {
    /// The errors which may occur for this client.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Get the URL for the endpoint for the client.
    ///
    /// This method adds the hostname for the client's target instance.
    fn rest_endpoint(&self, path: &str) -> Result<url::Url, APIError<Self::Error>>;

    /// Modifies the request.
    /// NOTE: This is done before adding the body (the final step), so it may be overwritten.
    fn modify_request(&self, request: Builder) -> Result<Builder, APIError<Self::Error>> {
        Ok(request)
    }
}

/// A trait represnting a client which includes a reqwest client.
#[cfg(feature = "reqwest")]
pub trait ReqwestAsyncClient: RestClient {
    /// Get the reqwest client.
    fn client(&self) -> &reqwest::Client;
}

/// A trait representing a client which includes a blocking reqwest client.
#[cfg(feature = "reqwest")]
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

#[cfg(feature = "reqwest")]
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
            headers.insert(key, value.clone());
        }

        // Add the body and return the response
        Ok(http_response.body(response.bytes()?)?)
    }
}

/// A trait representing an asynchronous client.
pub trait AsyncClient: RestClient {
    /// Send a REST query asynchronously.
    fn rest_async(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> impl std::future::Future<Output = Result<Response<Bytes>, APIError<Self::Error>>> + Send;
}

#[cfg(feature = "reqwest")]
impl<C> AsyncClient for C
where
    C: RestClient + ReqwestAsyncClient + Sync,
{
    async fn rest_async(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> Result<Response<Bytes>, APIError<Self::Error>> {
        // Send the request
        let response = self.client().execute(request.try_into()?).await?;

        // Construct the response builder
        let mut http_response = http::Response::builder()
            .status(response.status())
            .version(response.version());

        // Add headers
        let headers = http_response.headers_mut().unwrap();
        for (key, value) in response.headers() {
            headers.insert(key, value.clone());
        }

        // Add the body and return the response
        Ok(http_response.body(response.bytes().await?)?)
    }
}
