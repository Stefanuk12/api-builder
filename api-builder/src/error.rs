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

/// Errors that can occur when using API endpoints.
#[derive(thiserror::Error, Debug)]
pub enum APIError<E: std::error::Error + Send + Sync + 'static> {
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
impl<E: std::error::Error + Send + Sync + 'static> APIError<E> {
    /// Convert an `APIError<T>` to `APIError<E>`.
    pub fn from_api_error<T: std::error::Error + Send + Sync + 'static + Into<E>>(err: APIError<T>) -> APIError<E> {
        match err {
            APIError::Client(e) => APIError::Client(e.into()),
            APIError::HTTP(e) => APIError::HTTP(e),
            #[cfg(feature = "reqwest")]
            APIError::Reqwest(e) => APIError::Reqwest(e),
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
    pub fn from_any_api_error<T: std::error::Error + Send + Sync + 'static>(err: APIError<T>) -> APIError<E> {
        match err {
            APIError::Client(e) => APIError::Other(e.into()),
            APIError::HTTP(e) => APIError::HTTP(e),
            #[cfg(feature = "reqwest")]
            APIError::Reqwest(e) => APIError::Reqwest(e),
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

    /// Convert an error into `APIError<E>`.
    pub fn from_error<T: std::error::Error + Send + Sync + 'static + Into<E>>(err: T) -> APIError<E> {
        APIError::Client(err.into())
    }

    /// Convert any error into `APIError<E>` via the `Other` variant.
    pub fn from_any_error<T: std::error::Error + Send + Sync + 'static>(err: T) -> APIError<E> {
        APIError::Other(err.into())
    }
}