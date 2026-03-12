#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]

pub mod config;
pub mod error;
pub mod wallet;

pub use bdk_wallet;
pub use config::WalletConfig;
pub use error::BdkLibError;
pub use wallet::*;
