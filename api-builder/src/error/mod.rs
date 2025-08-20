pub mod body;
use core::fmt;
use std::{error::Error, fmt::Pointer};

pub use body::*;
pub mod header;
use bytes::Bytes;
pub use header::*;
pub mod http_client;
use http::Response;
pub use http_client::*;

pub trait APIClientError: std::error::Error + Send + Sync + 'static {}

pub struct APIError<E: APIClientError>(Box<Inner<E>>);
impl<E: APIClientError> APIError<E> {
    pub fn new(kind: APIErrorKind<E>) -> Self {
        Self(Box::new(Inner { kind }))
    }

    pub fn kind(&self) -> &APIErrorKind<E> {
        &self.0.kind
    }
}
impl<E: APIClientError, T> From<T> for APIError<E>
where
    T: Into<APIErrorKind<E>>,
{
    fn from(value: T) -> Self {
        Self::new(value.into())
    }
}
impl<E: APIClientError> fmt::Debug for APIError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl<E: APIClientError> fmt::Display for APIError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl<E: APIClientError> std::error::Error for APIError<E> {}

struct Inner<E: APIClientError> {
    kind: APIErrorKind<E>,
}

/// Errors that can occur when using API endpoints.
///
/// TODO: consider making this error struct lighter, there's a lot of bloat
#[derive(thiserror::Error, Debug)]
pub enum APIErrorKind<E: APIClientError> {
    /// The client encountered an error.
    #[error(transparent)]
    Client(E),
    /// There was an error with `http`.
    #[error(transparent)]
    Http(#[from] http::Error),
    // An error occured in a HTTP client.
    #[error(transparent)]
    HttpClient(#[from] HttpClientError),
    /// There was an error with the body.
    #[error(transparent)]
    Body(#[from] BodyError),
    /// There was an error with the headers.
    #[error(transparent)]
    Header(#[from] HeaderError),
    /// The server returned a error.
    #[error("the server returned an error")]
    Response(Response<Bytes>),
    /// URL parsing failed.
    #[error("failed to parse URL")]
    URL(#[from] url::ParseError),
    /// There was an unknown error.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
impl<E: APIClientError> APIErrorKind<E> {
    /// Convert an `APIErrorKind<T>` to `APIErrorKind<E>`.
    pub fn from_api_error<T: APIClientError + Into<E>>(err: APIErrorKind<T>) -> APIErrorKind<E> {
        match err {
            APIErrorKind::Client(e) => APIErrorKind::Client(e.into()),
            APIErrorKind::Http(e) => APIErrorKind::Http(e),
            APIErrorKind::HttpClient(e) => APIErrorKind::HttpClient(e),
            APIErrorKind::Body(e) => APIErrorKind::Body(e),
            APIErrorKind::Header(e) => APIErrorKind::Header(e),
            APIErrorKind::Response(e) => APIErrorKind::Response(e),
            APIErrorKind::URL(e) => APIErrorKind::URL(e),
            APIErrorKind::Other(e) => APIErrorKind::Other(e),
        }
    }

    /// Convert `Client` to `Other`.
    pub fn from_any_api_error<T: APIClientError>(err: APIErrorKind<T>) -> APIErrorKind<E> {
        match err {
            APIErrorKind::Client(e) => APIErrorKind::Other(e.into()),
            APIErrorKind::Http(e) => APIErrorKind::Http(e),
            APIErrorKind::HttpClient(e) => APIErrorKind::HttpClient(e),
            APIErrorKind::Body(e) => APIErrorKind::Body(e),
            APIErrorKind::Header(e) => APIErrorKind::Header(e),
            APIErrorKind::Response(e) => APIErrorKind::Response(e),
            APIErrorKind::URL(e) => APIErrorKind::URL(e),
            APIErrorKind::Other(e) => APIErrorKind::Other(e),
        }
    }

    /// Convert an error into `APIErrorKind<E>`.
    pub fn from_error<T: Error + Send + Sync + 'static + Into<E>>(err: T) -> APIErrorKind<E> {
        APIErrorKind::Client(err.into())
    }

    /// Convert any error into `APIErrorKind<E>` via the `Other` variant.
    pub fn from_any_error<T: Error + Send + Sync + 'static>(err: T) -> APIErrorKind<E> {
        APIErrorKind::Other(err.into())
    }
}

impl<E: APIClientError> From<E> for APIErrorKind<E> {
    fn from(value: E) -> Self {
        APIErrorKind::Client(value)
    }
}
impl<E: APIClientError> From<Response<Bytes>> for APIErrorKind<E> {
    fn from(value: Response<Bytes>) -> Self {
        Self::Response(value)
    }
}

macro_rules! impl_error_conv {
    ($variant:ident, $err:ty, $variant_2:ident, $err2:ty) => {
        impl<E: APIClientError> From<$err2> for APIErrorKind<E> {
            fn from(value: $err2) -> Self {
                Self::$variant(<$err>::$variant_2(value))
            }
        }
    };
}

impl_error_conv!(Body, BodyError, SerdeJson, serde_json::Error);
impl_error_conv!(Header, HeaderError, Parse, http::header::InvalidHeaderValue);
impl_error_conv!(Header, HeaderError, ToStr, http::header::ToStrError);

#[cfg(feature = "reqwest")]
impl_error_conv!(HttpClient, HttpClientError, Reqwest, reqwest::Error);
#[cfg(feature = "rquest")]
impl_error_conv!(HttpClient, HttpClientError, Rquest, rquest::Error);
#[cfg(target_arch = "wasm32")]
impl_error_conv!(HttpClient, HttpClientError, GlooNet, gloo_net::Error);
