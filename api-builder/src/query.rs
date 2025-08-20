// Dependencies
use std::future::Future;

use crate::{client::AsyncClient, error::APIError, Client};
use bytes::Bytes;
use http::{request::Builder, Response};

/// A trait which represents a query which may be made to a client.
pub trait Query<T, C>
where
    C: Client,
{
    /// Starts building the query request.
    fn request(&self, client: &C) -> Result<Builder, APIError<C::Error>>;

    /// Sends the request.
    fn send(&self, client: &C, request: Builder) -> Result<Response<Bytes>, APIError<C::Error>>;

    /// Finalises the request by returning the response.
    fn finalise(&self, response: Response<Bytes>) -> Result<T, APIError<C::Error>>;

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
    ) -> impl Future<Output = Result<Builder, APIError<C::Error>>> + Send;

    #[cfg(not(target_arch = "wasm32"))]
    /// Sends the request.
    fn send_async(
        &self,
        client: &C,
        request: Builder,
    ) -> impl Future<Output = Result<Response<Bytes>, APIError<C::Error>>> + Send;
    #[cfg(target_arch = "wasm32")]
    /// Sends the request.
    fn send_async(
        &self,
        client: &C,
        request: Builder,
    ) -> impl Future<Output = Result<Response<Bytes>, APIError<C::Error>>>;

    /// Finalises the request by returning the response.
    fn finalise_async(
        &self,
        response: Response<Bytes>,
    ) -> impl Future<Output = Result<T, APIError<C::Error>>> + Send;

    /// Perform the query asynchronously against the client.
    #[cfg(not(target_arch = "wasm32"))]
    fn query_async(&self, client: &C) -> impl Future<Output = Result<T, APIError<C::Error>>> + Send
    where
        C::Error: core::error::Error + Sync + Send + 'static;
    #[cfg(target_arch = "wasm32")]
    fn query_async(&self, client: &C) -> impl Future<Output = Result<T, APIError<C::Error>>>
    where
        C::Error: core::error::Error + Sync + Send + 'static;
}
