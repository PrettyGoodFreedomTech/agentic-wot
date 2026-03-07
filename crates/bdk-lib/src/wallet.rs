use std::fs;
use std::str::FromStr;

use bdk_esplora::esplora_client;
use bdk_esplora::EsploraAsyncExt;
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::bitcoin::{Address, Amount, Network};
use bdk_wallet::file_store::Store;
use bdk_wallet::{ChangeSet, KeychainKind, PersistedWallet, Wallet};
use bip39::Mnemonic;
use serde::{Deserialize, Serialize};

use crate::config::WalletConfig;
use crate::error::BdkLibError;

const STOP_GAP: usize = 5;
const PARALLEL_REQUESTS: usize = 5;
const DB_MAGIC: &[u8] = b"magic-carpet";

pub type WalletStore = Store<ChangeSet>;
pub type BdkWallet = PersistedWallet<WalletStore>;

#[derive(Debug, Serialize, Deserialize)]
pub struct DescriptorConfig {
    pub external: String,
    pub internal: String,
    pub network: String,
}

#[derive(Debug, Serialize)]
pub struct WalletCreationResult {
    pub mnemonic: String,
    pub external_descriptor: String,
    pub internal_descriptor: String,
    pub network: String,
}

#[derive(Debug, Serialize)]
pub struct WalletBalance {
    pub immature_sats: u64,
    pub trusted_pending_sats: u64,
    pub untrusted_pending_sats: u64,
    pub confirmed_sats: u64,
    pub total_sats: u64,
}

#[derive(Debug, Serialize)]
pub struct SendResult {
    pub txid: String,
    pub amount_sats: u64,
    pub fee_sats: u64,
}

#[derive(Debug, Serialize)]
pub struct TxInfo {
    pub txid: String,
    pub sent_sats: u64,
    pub received_sats: u64,
    pub fee_sats: Option<u64>,
    pub confirmed: bool,
    pub confirmation_height: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct AddressResult {
    pub address: String,
    pub index: u32,
}

fn derive_descriptors(
    mnemonic: &Mnemonic,
    network: Network,
) -> Result<(String, String), BdkLibError> {
    let seed = mnemonic.to_seed("");
    let xprv = Xpriv::new_master(network, &seed)
        .map_err(|e| BdkLibError::Wallet(format!("Failed to derive master key: {e}")))?;

    if network == Network::Bitcoin {
        Ok((
            format!("wpkh({xprv}/84'/0'/0'/0/*)"),
            format!("wpkh({xprv}/84'/0'/0'/1/*)"),
        ))
    } else {
        Ok((
            format!("wpkh({xprv}/84'/1'/0'/0/*)"),
            format!("wpkh({xprv}/84'/1'/0'/1/*)"),
        ))
    }
}

fn save_wallet_config(
    config: &WalletConfig,
    mnemonic: &str,
    external: &str,
    internal: &str,
) -> Result<(), BdkLibError> {
    fs::create_dir_all(&config.data_dir)?;

    let mnemonic_path = config.mnemonic_path();
    fs::write(&mnemonic_path, mnemonic)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&mnemonic_path, fs::Permissions::from_mode(0o600))?;
    }

    let desc_config = DescriptorConfig {
        external: external.to_string(),
        internal: internal.to_string(),
        network: config.network.to_string(),
    };
    let json = serde_json::to_string_pretty(&desc_config)
        .map_err(|e| BdkLibError::Persistence(e.to_string()))?;
    fs::write(config.descriptors_path(), json)?;

    Ok(())
}

fn load_descriptors(config: &WalletConfig) -> Result<DescriptorConfig, BdkLibError> {
    let path = config.descriptors_path();
    if !path.exists() {
        return Err(BdkLibError::WalletNotFound);
    }
    let data = fs::read_to_string(&path)?;
    serde_json::from_str(&data).map_err(|e| BdkLibError::Persistence(e.to_string()))
}

fn open_or_create_store(config: &WalletConfig) -> Result<WalletStore, BdkLibError> {
    let db_path = config.db_path();
    if db_path.exists() {
        let (store, _changeset) = Store::load(DB_MAGIC, &db_path)
            .map_err(|e| BdkLibError::Persistence(format!("Failed to load wallet DB: {e}")))?;
        Ok(store)
    } else {
        Store::create(DB_MAGIC, &db_path)
            .map_err(|e| BdkLibError::Persistence(format!("Failed to create wallet DB: {e}")))
    }
}

pub fn init_wallet(config: &WalletConfig) -> Result<WalletCreationResult, BdkLibError> {
    if config.mnemonic_path().exists() {
        return Err(BdkLibError::WalletAlreadyExists {
            path: config.data_dir.display().to_string(),
        });
    }

    let mnemonic = Mnemonic::generate(12).map_err(|e| BdkLibError::Mnemonic(e.to_string()))?;
    let (external, internal) = derive_descriptors(&mnemonic, config.network)?;
    let mnemonic_str = mnemonic.to_string();

    save_wallet_config(config, &mnemonic_str, &external, &internal)?;

    Ok(WalletCreationResult {
        mnemonic: mnemonic_str,
        external_descriptor: external,
        internal_descriptor: internal,
        network: config.network.to_string(),
    })
}

pub fn import_wallet(
    config: &WalletConfig,
    mnemonic_str: &str,
) -> Result<WalletCreationResult, BdkLibError> {
    if config.mnemonic_path().exists() {
        return Err(BdkLibError::WalletAlreadyExists {
            path: config.data_dir.display().to_string(),
        });
    }

    let mnemonic =
        Mnemonic::from_str(mnemonic_str).map_err(|e| BdkLibError::Mnemonic(e.to_string()))?;
    let (external, internal) = derive_descriptors(&mnemonic, config.network)?;
    let mnemonic_string = mnemonic.to_string();

    save_wallet_config(config, &mnemonic_string, &external, &internal)?;

    Ok(WalletCreationResult {
        mnemonic: mnemonic_string,
        external_descriptor: external,
        internal_descriptor: internal,
        network: config.network.to_string(),
    })
}

pub fn load_wallet(config: &WalletConfig) -> Result<(WalletStore, BdkWallet), BdkLibError> {
    let desc = load_descriptors(config)?;
    let mut db = open_or_create_store(config)?;

    let external = desc.external.clone();
    let internal = desc.internal.clone();

    let wallet_opt = Wallet::load()
        .descriptor(KeychainKind::External, Some(external.clone()))
        .descriptor(KeychainKind::Internal, Some(internal.clone()))
        .extract_keys()
        .check_network(config.network)
        .load_wallet(&mut db)
        .map_err(|e| BdkLibError::Wallet(e.to_string()))?;

    let wallet = match wallet_opt {
        Some(w) => w,
        None => Wallet::create(external, internal)
            .network(config.network)
            .create_wallet(&mut db)
            .map_err(|e| BdkLibError::Wallet(e.to_string()))?,
    };

    Ok((db, wallet))
}

pub fn next_address(
    wallet: &mut BdkWallet,
    db: &mut WalletStore,
) -> Result<AddressResult, BdkLibError> {
    let addr_info = wallet.next_unused_address(KeychainKind::External);
    wallet
        .persist(db)
        .map_err(|e| BdkLibError::Persistence(e.to_string()))?;
    Ok(AddressResult {
        address: addr_info.address.to_string(),
        index: addr_info.index,
    })
}

pub async fn sync_wallet(
    wallet: &mut BdkWallet,
    db: &mut WalletStore,
    esplora_url: &str,
) -> Result<(), BdkLibError> {
    let client = esplora_client::Builder::new(esplora_url)
        .build_async()
        .map_err(|e| BdkLibError::Esplora(e.to_string()))?;

    let request = wallet.start_full_scan().build();
    let update = client
        .full_scan(request, STOP_GAP, PARALLEL_REQUESTS)
        .await
        .map_err(|e| BdkLibError::Esplora(e.to_string()))?;

    wallet
        .apply_update(update)
        .map_err(|e| BdkLibError::Wallet(e.to_string()))?;
    wallet
        .persist(db)
        .map_err(|e| BdkLibError::Persistence(e.to_string()))?;

    Ok(())
}

pub fn get_balance(wallet: &BdkWallet) -> WalletBalance {
    let balance = wallet.balance();
    WalletBalance {
        immature_sats: balance.immature.to_sat(),
        trusted_pending_sats: balance.trusted_pending.to_sat(),
        untrusted_pending_sats: balance.untrusted_pending.to_sat(),
        confirmed_sats: balance.confirmed.to_sat(),
        total_sats: balance.total().to_sat(),
    }
}

pub async fn send(
    wallet: &mut BdkWallet,
    db: &mut WalletStore,
    esplora_url: &str,
    to: &str,
    amount_sats: u64,
) -> Result<SendResult, BdkLibError> {
    let address = Address::from_str(to)
        .map_err(|e| BdkLibError::Address(e.to_string()))?
        .require_network(wallet.network())
        .map_err(|e| BdkLibError::Address(e.to_string()))?;

    let mut tx_builder = wallet.build_tx();
    tx_builder.add_recipient(address.script_pubkey(), Amount::from_sat(amount_sats));

    let mut psbt = tx_builder
        .finish()
        .map_err(|e| BdkLibError::Transaction(e.to_string()))?;

    let finalized = wallet
        .sign(&mut psbt, Default::default())
        .map_err(|e| BdkLibError::Transaction(e.to_string()))?;

    if !finalized {
        return Err(BdkLibError::Transaction(
            "Failed to finalize transaction".into(),
        ));
    }

    let tx = psbt
        .extract_tx()
        .map_err(|e| BdkLibError::Transaction(e.to_string()))?;
    let txid = tx.compute_txid();

    let fee_sats = wallet
        .calculate_fee(&tx)
        .map(|f| f.to_sat())
        .unwrap_or(0);

    let client = esplora_client::Builder::new(esplora_url)
        .build_async()
        .map_err(|e| BdkLibError::Esplora(e.to_string()))?;

    client
        .broadcast(&tx)
        .await
        .map_err(|e| BdkLibError::Esplora(e.to_string()))?;

    wallet
        .persist(db)
        .map_err(|e| BdkLibError::Persistence(e.to_string()))?;

    Ok(SendResult {
        txid: txid.to_string(),
        amount_sats,
        fee_sats,
    })
}

pub fn list_transactions(wallet: &BdkWallet) -> Vec<TxInfo> {
    wallet
        .transactions()
        .map(|tx| {
            let txid = tx.tx_node.txid.to_string();
            let (sent, received) = wallet.sent_and_received(&tx.tx_node.tx);
            let fee = wallet
                .calculate_fee(&tx.tx_node.tx)
                .ok()
                .map(|f| f.to_sat());
            let (confirmed, height) = match &tx.chain_position {
                bdk_wallet::chain::ChainPosition::Confirmed { anchor, .. } => {
                    (true, Some(anchor.block_id.height))
                }
                bdk_wallet::chain::ChainPosition::Unconfirmed { .. } => (false, None),
            };
            TxInfo {
                txid,
                sent_sats: sent.to_sat(),
                received_sats: received.to_sat(),
                fee_sats: fee,
                confirmed,
                confirmation_height: height,
            }
        })
        .collect()
}
