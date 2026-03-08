#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]

pub mod client;
pub mod error;
pub mod types;

pub use client::PhoenixdClient;
pub use error::PhoenixdError;
pub use types::*;
