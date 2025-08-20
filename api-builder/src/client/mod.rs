use core::future::Future;

use bytes::Bytes;
use http::{Request, Response};
use url::Url;

use crate::APIError;

#[cfg(feature = "reqwest")]
import!(reqwest);
#[cfg(target_family = "wasm")]
#[cfg(not(feature = "reqwest"))]
import!(wasm);

/// A trait representing a client which can communicate with an instance via REST.
pub trait RestClient {
    /// The errors which may occur for this client.
    type Error;

    /// Get the URL for the endpoint for the client.
    ///
    /// This method adds the hostname for the client's target instance.
    fn rest_endpoint(&self, path: &str) -> Result<Url, APIError<Self::Error>>;
}

/// A trait representing a client.
pub trait Client: RestClient {
    /// Send a REST query.
    fn rest(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, APIError<Self::Error>>;
}

/// A trait representing an asynchronous client.
pub trait AsyncClient: RestClient {
    #[cfg(not(target_arch = "wasm32"))]
    /// Send a REST query asynchronously.
    fn rest_async(
        &self,
        request: Request<Vec<u8>>,
    ) -> impl Future<Output = Result<Response<Bytes>, APIError<Self::Error>>> + Send;

    #[cfg(target_arch = "wasm32")]
    /// Send a REST query asynchronously.
    fn rest_async(
        &self,
        request: Request<Vec<u8>>,
    ) -> impl Future<Output = Result<Response<Bytes>, APIError<Self::Error>>>;
}
