use std::path::PathBuf;

use bdk_wallet::bitcoin::Network;

pub struct WalletConfig {
    pub network: Network,
    pub esplora_url: String,
    pub data_dir: PathBuf,
}

impl WalletConfig {
    pub fn new(network: Network, esplora_url: &str) -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let network_str = match network {
            Network::Bitcoin => "mainnet",
            Network::Testnet => "testnet",
            Network::Signet => "signet",
            Network::Regtest => "regtest",
            _ => "unknown",
        };
        Self {
            network,
            esplora_url: esplora_url.to_string(),
            data_dir: home.join(".magic-carpet").join(network_str),
        }
    }

    pub fn mnemonic_path(&self) -> PathBuf {
        self.data_dir.join("mnemonic")
    }

    pub fn descriptors_path(&self) -> PathBuf {
        self.data_dir.join("descriptors.json")
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("wallet.db")
    }
}
