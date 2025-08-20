/// Errors that can occur when adding headers.
#[derive(thiserror::Error, Debug)]
pub enum HeaderError {
    #[error(transparent)]
    Parse(#[from] http::header::InvalidHeaderValue),
    #[error("missing expected header: {0}")]
    MissingHeader(&'static str),
    #[error(transparent)]
    ToStr(#[from] http::header::ToStrError),
    /// There was invalid header data.
    #[error("invalid header data, missing name")]
    MissingHeaderName,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
