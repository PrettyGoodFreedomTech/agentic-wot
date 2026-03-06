use thiserror::Error;

#[derive(Error, Debug)]
pub enum BdkLibError {
    #[error("Wallet error: {0}")]
    Wallet(String),

    #[error("Esplora error: {0}")]
    Esplora(String),

    #[error("Persistence error: {0}")]
    Persistence(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Mnemonic error: {0}")]
    Mnemonic(String),

    #[error("Address error: {0}")]
    Address(String),

    #[error("Wallet already exists at {path}")]
    WalletAlreadyExists { path: String },

    #[error("Wallet not found — run init first")]
    WalletNotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
