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
    Other(#[from] anyhow::Error)
}

/// Errors that can occur when adding headers.
#[derive(thiserror::Error, Debug)]
pub enum HeaderError {
    #[error(transparent)]
    Parse(#[from] http::header::InvalidHeaderValue),
    #[error(transparent)]
    Other(#[from] anyhow::Error)
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
    Other(anyhow::Error)
}