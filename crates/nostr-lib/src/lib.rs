#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]

pub mod error;
pub mod filters;
pub mod profile;
pub mod service;
pub mod types;
pub mod zap;

pub use error::NostrLibError;
pub use service::NostrService;
pub use types::*;
