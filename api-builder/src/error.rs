/// Errors that can occur when building the body.
#[derive(thiserror::Error, Debug)]
pub enum BodyError {
    #[error("there was insufficient data to build the body")]
    InsufficientData,
    #[error("there was an error building the body")]
    Build,
    #[error("failed to deserialize")]
    Deserialize,
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Errors that can occur when adding headers.
#[derive(thiserror::Error, Debug)]
pub enum HeaderError {
    #[error(transparent)]
    Parse(#[from] http::header::InvalidHeaderValue),
    #[error("missing expected header: {0}")]
    MissingHeader(&'static str),
    #[error(transparent)]
    ToStr(#[from] http::header::ToStrError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub trait APIClientError: std::error::Error + Send + Sync + 'static {}

/// Errors that can occur when using API endpoints.
#[derive(thiserror::Error, Debug)]
pub enum APIError<E: APIClientError> {
    /// The client encountered an error.
    #[error(transparent)]
    Client(E),
    /// There was an error with `http`.
    #[error(transparent)]
    HTTP(#[from] http::Error),
    /// There was an error with `reqwest`.
    #[cfg(feature = "reqwest")]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// There was an error with `rquest`.
    #[cfg(feature = "rquest")]
    #[error(transparent)]
    Rquest(#[from] rquest::Error),
    /// There was an error with `gloo-net`.
    #[cfg(target_arch = "wasm32")]
    #[error(transparent)]
    GlooNet(#[from] gloo_net::Error),
    /// There was an error with the body.
    #[error(transparent)]
    Body(#[from] BodyError),
    /// There was an error with the headers.
    #[error(transparent)]
    Header(#[from] HeaderError),
    /// There was invalid header data.
    #[error("invalid header data, missing name")]
    MissingHeaderName,
    /// The server returned a error.
    #[error("the server returned an error")]
    Response(http::Response<bytes::Bytes>),
    /// URL parsing failed.
    #[error("failed to parse URL")]
    URL(#[from] url::ParseError),
    /// There was an unknown error.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
impl<E: APIClientError> APIError<E> {
    /// Convert an `APIError<T>` to `APIError<E>`.
    pub fn from_api_error<T: APIClientError + Into<E>>(err: APIError<T>) -> APIError<E> {
        match err {
            APIError::Client(e) => APIError::Client(e.into()),
            APIError::HTTP(e) => APIError::HTTP(e),
            #[cfg(feature = "reqwest")]
            APIError::Reqwest(e) => APIError::Reqwest(e),
            #[cfg(feature = "rquest")]
            APIError::Rquest(e) => APIError::Rquest(e),
            #[cfg(target_arch = "wasm32")]
            APIError::GlooNet(e) => APIError::GlooNet(e),
            APIError::Body(e) => APIError::Body(e),
            APIError::Header(e) => APIError::Header(e),
            APIError::MissingHeaderName => APIError::MissingHeaderName,
            APIError::Response(e) => APIError::Response(e),
            APIError::URL(e) => APIError::URL(e),
            APIError::Other(e) => APIError::Other(e),
        }
    }

    /// Convert `Client` to `Other`.
    pub fn from_any_api_error<T: APIClientError>(err: APIError<T>) -> APIError<E> {
        match err {
            APIError::Client(e) => APIError::Other(e.into()),
            APIError::HTTP(e) => APIError::HTTP(e),
            #[cfg(feature = "reqwest")]
            APIError::Reqwest(e) => APIError::Reqwest(e),
            #[cfg(feature = "rquest")]
            APIError::Rquest(e) => APIError::Rquest(e),
            #[cfg(target_arch = "wasm32")]
            APIError::GlooNet(e) => APIError::GlooNet(e),
            APIError::Body(e) => APIError::Body(e),
            APIError::Header(e) => APIError::Header(e),
            APIError::MissingHeaderName => APIError::MissingHeaderName,
            APIError::Response(e) => APIError::Response(e),
            APIError::URL(e) => APIError::URL(e),
            APIError::Other(e) => APIError::Other(e),
        }
    }

    /// Convert any error into `APIError<E>` via the `Other` variant.
    pub fn from_any_error<T: std::error::Error + Send + Sync + 'static>(err: T) -> APIError<E> {
        APIError::Other(err.into())
    }
}

impl<E: APIClientError> From<E> for APIError<E> {
    fn from(value: E) -> Self {
        APIError::Client(value)
    }
}
impl<E: APIClientError> From<http::Response<bytes::Bytes>> for APIError<E> {
    fn from(value: http::Response<bytes::Bytes>) -> Self {
        Self::Response(value)
    }
}

macro_rules! impl_error_conv {
    ($variant:ident, $err:ty, $variant_2:ident, $err2:ty) => {
        impl<E: APIClientError> From<$err2> for APIError<E> {
            fn from(value: $err2) -> Self {
                Self::$variant(<$err>::$variant_2(value))
            }
        }
    };
}

impl_error_conv!(Body, BodyError, SerdeJson, serde_json::Error);
impl_error_conv!(Header, HeaderError, Parse, http::header::InvalidHeaderValue);
impl_error_conv!(Header, HeaderError, ToStr, http::header::ToStrError);