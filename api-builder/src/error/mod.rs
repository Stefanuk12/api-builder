use core::{
    error::Error,
    fmt::{self, Pointer},
};

use bytes::Bytes;
use http::Response;

import!(body, header, http_client);

pub struct APIError<E>(Box<Inner<E>>);
impl<E> APIError<E> {
    pub fn new(kind: APIErrorKind<E>) -> Self {
        Self(Box::new(Inner { kind }))
    }

    pub fn kind(&self) -> &APIErrorKind<E> {
        &self.0.kind
    }
}
impl<E, T> From<T> for APIError<E>
where
    T: Into<APIErrorKind<E>>,
{
    fn from(value: T) -> Self {
        Self::new(value.into())
    }
}
impl<E> fmt::Debug for APIError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl<E> fmt::Display for APIError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl<E> core::error::Error for APIError<E> {}

struct Inner<E> {
    kind: APIErrorKind<E>,
}

/// Errors that can occur when using API endpoints.
///
/// TODO: consider making this error struct lighter, there's a lot of bloat
#[derive(Debug, thiserror::Error)]
pub enum APIErrorKind<E> {
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
impl<E> APIErrorKind<E> {
    /// Convert an `APIErrorKind<T>` to `APIErrorKind<E>`.
    pub fn from_api_error<T: Into<E>>(err: APIErrorKind<T>) -> APIErrorKind<E> {
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
    pub fn from_any_api_error<T: core::error::Error + Sync + Send + 'static>(
        err: APIErrorKind<T>,
    ) -> APIErrorKind<E> {
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
macro_rules! impl_error_conv {
    ($variant:ident, $err:ty, $variant_2:ident, $err2:ty) => {
        impl<E> From<$err2> for APIErrorKind<E> {
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
#[cfg(target_arch = "wasm32")]
impl_error_conv!(HttpClient, HttpClientError, GlooNet, gloo_net::Error);
