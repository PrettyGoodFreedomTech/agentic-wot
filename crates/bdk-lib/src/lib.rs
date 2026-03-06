pub mod config;
pub mod error;
pub mod wallet;

pub use bdk_wallet;
pub use config::WalletConfig;
pub use error::BdkLibError;
pub use wallet::*;
