// Errors that can occur from HTTP clients.
#[derive(Debug, thiserror::Error)]
pub enum HttpClientError {
    /// There was an error with `reqwest`.
    #[cfg(feature = "reqwest")]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    /// There was an error with `rquest`.
    /// There was an error with `gloo-net`.
    #[cfg(target_arch = "wasm32")]
    #[error(transparent)]
    GlooNet(#[from] gloo_net::Error),
    /// There was an unknown error from an unknown client.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
