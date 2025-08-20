/// Errors that can occur when building the body.
#[derive(Debug, thiserror::Error)]
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
