/// Errors that can occur when adding headers.
#[derive(Debug, thiserror::Error)]
pub enum HeaderError {
    #[error(transparent)]
    Parse(#[from] http::header::InvalidHeaderValue),
    #[error("missing expected header: {0}")]
    MissingHeader(&'static str),
    #[error(transparent)]
    ToStr(#[from] http::header::ToStrError),
    #[error("invalid header data, missing name")]
    MissingHeaderName,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
