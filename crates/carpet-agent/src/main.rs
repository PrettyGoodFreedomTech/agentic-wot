use std::io::{self, Write};
use std::sync::Arc;

use anyhow::Result;
use rig::client::CompletionClient;
use rig::completion::{Prompt, ToolDefinition};
use rig::tool::Tool;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;

use bdk_lib::bdk_wallet;
use bdk_lib::{BdkWallet, WalletConfig, WalletStore};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct ToolError(String);

fn wallet_config() -> WalletConfig {
    let network_str = std::env::var("BDK_NETWORK").unwrap_or_else(|_| "regtest".into());
    let network = match network_str.as_str() {
        "mainnet" | "bitcoin" => bdk_wallet::bitcoin::Network::Bitcoin,
        "testnet" => bdk_wallet::bitcoin::Network::Testnet,
        "signet" => bdk_wallet::bitcoin::Network::Signet,
        _ => bdk_wallet::bitcoin::Network::Regtest,
    };
    let esplora =
        std::env::var("BDK_ESPLORA_URL").unwrap_or_else(|_| "http://localhost:3002".into());
    WalletConfig::new(network, &esplora)
}

type SharedWallet = Arc<Mutex<Option<(WalletStore, BdkWallet)>>>;

async fn ensure_loaded(shared: &SharedWallet) -> Result<(), ToolError> {
    let mut guard = shared.lock().await;
    if guard.is_none() {
        let config = wallet_config();
        let (db, wallet) =
            bdk_lib::load_wallet(&config).map_err(|e| ToolError(e.to_string()))?;
        *guard = Some((db, wallet));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// BDK Tools
// ---------------------------------------------------------------------------

struct CreateWalletTool;

#[derive(Deserialize)]
struct CreateWalletArgs {
    #[serde(default)]
    mnemonic: Option<String>,
}

impl Tool for CreateWalletTool {
    const NAME: &'static str = "create_bitcoin_wallet";
    type Error = ToolError;
    type Args = CreateWalletArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Create or import a Bitcoin wallet. Omit mnemonic to generate a new one."
                .into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "mnemonic": {
                        "type": "string",
                        "description": "Optional 12-word mnemonic to import. Leave empty to generate."
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<serde_json::Value, ToolError> {
        let config = wallet_config();
        let result = match args.mnemonic {
            Some(m) => bdk_lib::import_wallet(&config, &m),
            None => bdk_lib::init_wallet(&config),
        }
        .map_err(|e| ToolError(e.to_string()))?;
        serde_json::to_value(result).map_err(|e| ToolError(e.to_string()))
    }
}

struct GetAddressTool {
    wallet: SharedWallet,
}

#[derive(Deserialize)]
struct EmptyArgs {}

impl Tool for GetAddressTool {
    const NAME: &'static str = "get_bitcoin_address";
    type Error = ToolError;
    type Args = EmptyArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Get the next unused Bitcoin receive address".into(),
            parameters: json!({"type": "object", "properties": {}}),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<serde_json::Value, ToolError> {
        ensure_loaded(&self.wallet).await?;
        let mut guard = self.wallet.lock().await;
        let (db, wallet) = guard.as_mut().ok_or(ToolError("Wallet not loaded".into()))?;
        let addr = bdk_lib::next_address(wallet, db).map_err(|e| ToolError(e.to_string()))?;
        serde_json::to_value(addr).map_err(|e| ToolError(e.to_string()))
    }
}

struct SyncWalletTool {
    wallet: SharedWallet,
}

impl Tool for SyncWalletTool {
    const NAME: &'static str = "sync_bitcoin_wallet";
    type Error = ToolError;
    type Args = EmptyArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Sync the Bitcoin wallet with the blockchain (Esplora)".into(),
            parameters: json!({"type": "object", "properties": {}}),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<serde_json::Value, ToolError> {
        ensure_loaded(&self.wallet).await?;
        let config = wallet_config();
        let mut guard = self.wallet.lock().await;
        let (db, wallet) = guard.as_mut().ok_or(ToolError("Wallet not loaded".into()))?;
        bdk_lib::sync_wallet(wallet, db, &config.esplora_url)
            .await
            .map_err(|e| ToolError(e.to_string()))?;
        let balance = bdk_lib::get_balance(wallet);
        serde_json::to_value(json!({"synced": true, "balance": balance}))
            .map_err(|e| ToolError(e.to_string()))
    }
}

struct GetBalanceTool {
    wallet: SharedWallet,
}

impl Tool for GetBalanceTool {
    const NAME: &'static str = "get_bitcoin_balance";
    type Error = ToolError;
    type Args = EmptyArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Get the Bitcoin wallet balance (confirmed, pending, total)".into(),
            parameters: json!({"type": "object", "properties": {}}),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<serde_json::Value, ToolError> {
        ensure_loaded(&self.wallet).await?;
        let guard = self.wallet.lock().await;
        let (_, wallet) = guard.as_ref().ok_or(ToolError("Wallet not loaded".into()))?;
        let balance = bdk_lib::get_balance(wallet);
        serde_json::to_value(balance).map_err(|e| ToolError(e.to_string()))
    }
}

struct SendBitcoinTool {
    wallet: SharedWallet,
}

#[derive(Deserialize)]
struct SendArgs {
    to: String,
    amount_sats: u64,
}

impl Tool for SendBitcoinTool {
    const NAME: &'static str = "send_bitcoin";
    type Error = ToolError;
    type Args = SendArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Send Bitcoin to an address".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "to": {"type": "string", "description": "Destination Bitcoin address"},
                    "amount_sats": {"type": "integer", "description": "Amount in satoshis"}
                },
                "required": ["to", "amount_sats"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<serde_json::Value, ToolError> {
        ensure_loaded(&self.wallet).await?;
        let config = wallet_config();
        let mut guard = self.wallet.lock().await;
        let (db, wallet) = guard.as_mut().ok_or(ToolError("Wallet not loaded".into()))?;
        let result = bdk_lib::send(wallet, db, &config.esplora_url, &args.to, args.amount_sats)
            .await
            .map_err(|e| ToolError(e.to_string()))?;
        serde_json::to_value(result).map_err(|e| ToolError(e.to_string()))
    }
}

struct ListTransactionsTool {
    wallet: SharedWallet,
}

impl Tool for ListTransactionsTool {
    const NAME: &'static str = "list_bitcoin_transactions";
    type Error = ToolError;
    type Args = EmptyArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "List Bitcoin wallet transactions".into(),
            parameters: json!({"type": "object", "properties": {}}),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<serde_json::Value, ToolError> {
        ensure_loaded(&self.wallet).await?;
        let guard = self.wallet.lock().await;
        let (_, wallet) = guard.as_ref().ok_or(ToolError("Wallet not loaded".into()))?;
        let txs = bdk_lib::list_transactions(wallet);
        serde_json::to_value(json!({"count": txs.len(), "transactions": txs}))
            .map_err(|e| ToolError(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// PhoenixD Tools
// ---------------------------------------------------------------------------

struct GetNodeInfoTool;

impl Tool for GetNodeInfoTool {
    const NAME: &'static str = "get_lightning_node_info";
    type Error = ToolError;
    type Args = EmptyArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Get Lightning node info from PhoenixD".into(),
            parameters: json!({"type": "object", "properties": {}}),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<serde_json::Value, ToolError> {
        let client =
            phoenixd_lib::PhoenixdClient::from_env().map_err(|e| ToolError(e.to_string()))?;
        let info = client
            .node_info()
            .await
            .map_err(|e| ToolError(e.to_string()))?;
        serde_json::to_value(info).map_err(|e| ToolError(e.to_string()))
    }
}

struct GetLightningBalanceTool;

impl Tool for GetLightningBalanceTool {
    const NAME: &'static str = "get_lightning_balance";
    type Error = ToolError;
    type Args = EmptyArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Get Lightning wallet balance from PhoenixD".into(),
            parameters: json!({"type": "object", "properties": {}}),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<serde_json::Value, ToolError> {
        let client =
            phoenixd_lib::PhoenixdClient::from_env().map_err(|e| ToolError(e.to_string()))?;
        let balance = client
            .get_balance()
            .await
            .map_err(|e| ToolError(e.to_string()))?;
        serde_json::to_value(balance).map_err(|e| ToolError(e.to_string()))
    }
}

struct CreateInvoiceTool;

#[derive(Deserialize)]
struct CreateInvoiceArgs {
    amount_sats: u64,
    description: String,
}

impl Tool for CreateInvoiceTool {
    const NAME: &'static str = "create_lightning_invoice";
    type Error = ToolError;
    type Args = CreateInvoiceArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Create a Lightning BOLT11 invoice via PhoenixD".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "amount_sats": {"type": "integer", "description": "Amount in satoshis"},
                    "description": {"type": "string", "description": "Invoice description"}
                },
                "required": ["amount_sats", "description"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<serde_json::Value, ToolError> {
        let client =
            phoenixd_lib::PhoenixdClient::from_env().map_err(|e| ToolError(e.to_string()))?;
        let invoice = client
            .create_invoice(args.amount_sats, &args.description, None)
            .await
            .map_err(|e| ToolError(e.to_string()))?;
        serde_json::to_value(invoice).map_err(|e| ToolError(e.to_string()))
    }
}

struct PayInvoiceTool;

#[derive(Deserialize)]
struct PayInvoiceArgs {
    bolt11: String,
}

impl Tool for PayInvoiceTool {
    const NAME: &'static str = "pay_lightning_invoice";
    type Error = ToolError;
    type Args = PayInvoiceArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Pay a Lightning BOLT11 invoice via PhoenixD".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "bolt11": {"type": "string", "description": "BOLT11 invoice string"}
                },
                "required": ["bolt11"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<serde_json::Value, ToolError> {
        let client =
            phoenixd_lib::PhoenixdClient::from_env().map_err(|e| ToolError(e.to_string()))?;
        let result = client
            .pay_invoice(&args.bolt11)
            .await
            .map_err(|e| ToolError(e.to_string()))?;
        serde_json::to_value(result).map_err(|e| ToolError(e.to_string()))
    }
}

struct ListChannelsTool;

impl Tool for ListChannelsTool {
    const NAME: &'static str = "list_lightning_channels";
    type Error = ToolError;
    type Args = EmptyArgs;
    type Output = serde_json::Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "List Lightning channels from PhoenixD".into(),
            parameters: json!({"type": "object", "properties": {}}),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<serde_json::Value, ToolError> {
        let client =
            phoenixd_lib::PhoenixdClient::from_env().map_err(|e| ToolError(e.to_string()))?;
        let channels = client
            .list_channels()
            .await
            .map_err(|e| ToolError(e.to_string()))?;
        serde_json::to_value(json!({"count": channels.len(), "channels": channels}))
            .map_err(|e| ToolError(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "carpet_agent=info".into()),
        )
        .init();

    let api_key =
        std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");

    let shared_wallet: SharedWallet = Arc::new(Mutex::new(None));

    let client: rig::providers::anthropic::Client =
        rig::providers::anthropic::Client::new(&api_key)
            .expect("Failed to create Anthropic client");

    let agent = client
        .agent("claude-sonnet-4-20250514")
        .preamble(
            "You are Magic Carpet, a Bitcoin and Lightning Network assistant. \
             You can manage on-chain Bitcoin wallets (create, sync, get addresses, send, check balance) \
             and Lightning payments via PhoenixD (node info, balance, create/pay invoices, list channels). \
             Always sync the wallet before reporting balances. \
             Report amounts in both satoshis and BTC when relevant.",
        )
        .tool(CreateWalletTool)
        .tool(GetAddressTool {
            wallet: shared_wallet.clone(),
        })
        .tool(SyncWalletTool {
            wallet: shared_wallet.clone(),
        })
        .tool(GetBalanceTool {
            wallet: shared_wallet.clone(),
        })
        .tool(SendBitcoinTool {
            wallet: shared_wallet.clone(),
        })
        .tool(ListTransactionsTool {
            wallet: shared_wallet,
        })
        .tool(GetNodeInfoTool)
        .tool(GetLightningBalanceTool)
        .tool(CreateInvoiceTool)
        .tool(PayInvoiceTool)
        .tool(ListChannelsTool)
        .build();

    println!("Magic Carpet - Bitcoin & Lightning Agent");
    println!("Type your request (Ctrl+D to exit):\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("> ");
        stdout.flush()?;

        let mut input = String::new();
        if stdin.read_line(&mut input)? == 0 {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match agent.prompt(input).await {
            Ok(response) => println!("\n{response}\n"),
            Err(e) => eprintln!("\nError: {e}\n"),
        }
    }

    Ok(())
}
