// Errors that can occur from HTTP clients.
#[derive(thiserror::Error, Debug)]
pub enum HttpClientError {
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
    /// There was an unknown error from an unknown client.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
