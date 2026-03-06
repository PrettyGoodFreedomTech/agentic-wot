use thiserror::Error;

#[derive(Error, Debug)]
pub enum PhoenixdError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error: {message}")]
    Api { message: String },

    #[error("Deserialization error: {0}")]
    Deserialize(String),
}
