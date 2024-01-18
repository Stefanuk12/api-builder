// Dependencies
use crate::{client::AsyncClient, error::APIError};
use bytes::Bytes;

/// A trait which represents a query which may be made to a client.
pub trait Query<T, C>
where
    C: crate::client::Client,
{
    /// Starts building the query request.
    fn request(&self, client: &C) -> Result<http::request::Builder, APIError<C::Error>>;

    /// Sends the request.
    fn send(
        &self,
        client: &C,
        request: http::request::Builder,
    ) -> Result<http::Response<Bytes>, APIError<C::Error>>;

    /// Finalises the request by returning the response.
    fn finalise(&self, response: http::Response<Bytes>) -> Result<T, APIError<C::Error>>;

    /// Perform the query against the client.
    fn query(&self, client: &C) -> Result<T, APIError<C::Error>>;
}

/// A trait which represents an asynchronous query which may be made to a client.
pub trait AsyncQuery<T, C>
where
    C: AsyncClient,
{
    /// Starts building the query request.
    fn request_async(
        &self,
        client: &C,
    ) -> impl std::future::Future<Output = Result<http::request::Builder, APIError<C::Error>>> + Send;

    /// Sends the request.
    fn send_async(
        &self,
        client: &C,
        request: http::request::Builder,
    ) -> impl std::future::Future<Output = Result<http::Response<Bytes>, APIError<C::Error>>> + Send;

    /// Finalises the request by returning the response.
    fn finalise_async(
        &self,
        response: http::Response<Bytes>,
    ) -> impl std::future::Future<Output = Result<T, APIError<C::Error>>> + Send;

    /// Perform the query asynchronously against the client.
    fn query_async(
        &self,
        client: &C,
    ) -> impl std::future::Future<Output = Result<T, APIError<C::Error>>> + Send;
}
