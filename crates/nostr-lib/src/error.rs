use thiserror::Error;

#[derive(Error, Debug)]
pub enum NostrLibError {
    #[error("Relay connection failed: {url}")]
    RelayConnection { url: String },

    #[error("Event publish failed: {reason}")]
    PublishFailed { reason: String },

    #[error("Protocol error: {0}")]
    Protocol(#[from] dcosl_core::DcoslError),

    #[error("Zap error: {reason}")]
    Zap { reason: String },

    #[error("Profile not found for pubkey: {pubkey}")]
    ProfileNotFound { pubkey: String },

    #[error("Missing lud16 in profile for pubkey: {pubkey}")]
    MissingLud16 { pubkey: String },

    #[error("LNURL error: {reason}")]
    Lnurl { reason: String },

    #[error("PhoenixD error: {0}")]
    Phoenixd(#[from] phoenixd_lib::PhoenixdError),

    #[error("Nostr SDK error: {0}")]
    Sdk(String),
}
