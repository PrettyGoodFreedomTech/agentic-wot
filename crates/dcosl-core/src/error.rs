use thiserror::Error;

/// Protocol-level errors for DCoSL operations.
///
/// These cover data validation issues — not network or CLI errors.
#[derive(Error, Debug)]
pub enum DcoslError {
    #[error("Invalid coordinate format: {input} — expected kind:pubkey:d-tag")]
    InvalidCoordinate { input: String },

    #[error("Header missing d-tag (required for addressable events)")]
    HeaderMissingDTag,

    #[error("Invalid event ID: {id}")]
    InvalidEventId { id: String },

    #[error("Invalid public key: {pubkey}")]
    InvalidPubkey { pubkey: String },
}

impl DcoslError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidCoordinate { .. } => "INVALID_COORDINATE",
            Self::HeaderMissingDTag => "HEADER_MISSING_D_TAG",
            Self::InvalidEventId { .. } => "INVALID_EVENT_ID",
            Self::InvalidPubkey { .. } => "INVALID_PUBKEY",
        }
    }

    pub fn retryable(&self) -> bool {
        false
    }
}
