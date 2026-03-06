use std::process;

use agcli::{
    AgentCli, Command, CommandError, CommandOutput, CommandRequest, ErrorEnvelope,
    ExecutionContext, NextAction,
};
use bdk_lib::{BdkLibError, WalletConfig};
use serde_json::json;

fn parse_bool_flag(req: &CommandRequest<'_>, name: &str) -> Result<bool, CommandError> {
    match req.flag(name) {
        None => Ok(false),
        Some("true") => Ok(true),
        Some(other) => Err(CommandError::new(
            format!("--{name} is a boolean flag, got unexpected value: {other}"),
            "INVALID_ARGS",
            format!("Use --{name} without a value, or remove it"),
        )),
    }
}

fn err(e: BdkLibError) -> CommandError {
    let code = match &e {
        BdkLibError::Wallet(_) => "WALLET_ERROR",
        BdkLibError::Esplora(_) => "ESPLORA_ERROR",
        BdkLibError::Persistence(_) => "PERSISTENCE_ERROR",
        BdkLibError::Transaction(_) => "TX_ERROR",
        BdkLibError::Mnemonic(_) => "MNEMONIC_ERROR",
        BdkLibError::Address(_) => "ADDRESS_ERROR",
        BdkLibError::WalletAlreadyExists { .. } => "WALLET_EXISTS",
        BdkLibError::WalletNotFound => "WALLET_NOT_FOUND",
        BdkLibError::Io(_) => "IO_ERROR",
    };
    let fix = match &e {
        BdkLibError::WalletNotFound => "Run `bdk-agcli init --generate` first".to_string(),
        BdkLibError::WalletAlreadyExists { path } => {
            format!("Wallet already exists at {path}. Delete to reinit.")
        }
        BdkLibError::Esplora(_) => "Check that Esplora is running: `just infra-up`".to_string(),
        _ => "Check error details".to_string(),
    };
    CommandError::new(e.to_string(), code, fix)
}

fn resolve_config() -> WalletConfig {
    let network_str = std::env::var("BDK_NETWORK").unwrap_or_else(|_| "regtest".into());
    let network = match network_str.as_str() {
        "mainnet" | "bitcoin" => bdk_lib::bdk_wallet::bitcoin::Network::Bitcoin,
        "testnet" => bdk_lib::bdk_wallet::bitcoin::Network::Testnet,
        "signet" => bdk_lib::bdk_wallet::bitcoin::Network::Signet,
        _ => bdk_lib::bdk_wallet::bitcoin::Network::Regtest,
    };
    let esplora =
        std::env::var("BDK_ESPLORA_URL").unwrap_or_else(|_| "http://localhost:3002".into());
    WalletConfig::new(network, &esplora)
}

fn init_command() -> Command {
    Command::new("init", "Generate or import a wallet mnemonic and descriptors")
        .usage("bdk-agcli init --generate | --import=<mnemonic>")
        .sync_handler(|req: &CommandRequest<'_>, _ctx: &mut ExecutionContext| {
            let generate = parse_bool_flag(req, "generate")?;
            let import = req.flag("import").map(String::from);

            if generate && import.is_some() {
                return Err(CommandError::new(
                    "--generate and --import are mutually exclusive",
                    "INVALID_ARGS",
                    "Use either --generate or --import, not both",
                ));
            }

            if !generate && import.is_none() {
                return Err(CommandError::new(
                    "Specify --generate or --import",
                    "MISSING_ARG",
                    "Use --generate to create a new wallet, or --import=<mnemonic>",
                )
                .next_actions(vec![NextAction::new(
                    "bdk-agcli init --generate",
                    "Generate new wallet",
                )]));
            }

            let config = resolve_config();

            let result = if generate {
                bdk_lib::init_wallet(&config).map_err(err)?
            } else {
                let mnemonic = import.unwrap();
                bdk_lib::import_wallet(&config, &mnemonic).map_err(err)?
            };

            Ok(CommandOutput::new(json!(result)).next_actions(vec![
                NextAction::new("bdk-agcli address", "Get a receive address"),
                NextAction::new("bdk-agcli sync", "Sync wallet with Esplora"),
            ]))
        })
}

fn address_command() -> Command {
    Command::new("address", "Get next unused receive address")
        .usage("bdk-agcli address")
        .sync_handler(
            |_req: &CommandRequest<'_>, _ctx: &mut ExecutionContext| {
                let config = resolve_config();
                let (mut db, mut wallet) = bdk_lib::load_wallet(&config).map_err(err)?;
                let addr = bdk_lib::next_address(&mut wallet, &mut db).map_err(err)?;

                Ok(CommandOutput::new(json!(addr)).next_actions(vec![
                    NextAction::new(
                        format!("just fund-bdk {}", addr.address),
                        "Fund this address (regtest)",
                    ),
                    NextAction::new("bdk-agcli sync", "Sync wallet after funding"),
                ]))
            },
        )
}

fn sync_command() -> Command {
    Command::new("sync", "Sync wallet with Esplora")
        .usage("bdk-agcli sync")
        .handler(
            |_req: &CommandRequest<'_>, _ctx: &mut ExecutionContext| {
                Box::pin(async move {
                    let config = resolve_config();
                    let (mut db, mut wallet) = bdk_lib::load_wallet(&config).map_err(err)?;

                    bdk_lib::sync_wallet(&mut wallet, &mut db, &config.esplora_url)
                        .await
                        .map_err(err)?;

                    let balance = bdk_lib::get_balance(&wallet);
                    Ok(CommandOutput::new(json!({
                        "synced": true,
                        "balance": balance,
                    }))
                    .next_actions(vec![
                        NextAction::new("bdk-agcli balance", "View balance"),
                        NextAction::new("bdk-agcli list-tx", "List transactions"),
                    ]))
                })
            },
        )
}

fn balance_command() -> Command {
    Command::new("balance", "Show wallet balance")
        .usage("bdk-agcli balance")
        .sync_handler(
            |_req: &CommandRequest<'_>, _ctx: &mut ExecutionContext| {
                let config = resolve_config();
                let (_db, wallet) = bdk_lib::load_wallet(&config).map_err(err)?;
                let balance = bdk_lib::get_balance(&wallet);

                Ok(CommandOutput::new(json!(balance)).next_actions(vec![
                    NextAction::new("bdk-agcli sync", "Sync first if stale"),
                    NextAction::new(
                        "bdk-agcli send --to <addr> --amount <sats>",
                        "Send BTC",
                    ),
                ]))
            },
        )
}

fn send_command() -> Command {
    Command::new("send", "Send BTC to an address")
        .usage("bdk-agcli send --to <address> --amount <sats>")
        .handler(
            |req: &CommandRequest<'_>, _ctx: &mut ExecutionContext| {
                let to = req.flag("to").map(String::from);
                let amount_str = req.flag("amount").map(String::from);

                Box::pin(async move {
                    let to = to.ok_or_else(|| {
                        CommandError::new(
                            "--to is required",
                            "MISSING_ARG",
                            "Provide --to=<address>",
                        )
                    })?;
                    let amount_sats: u64 = amount_str
                        .ok_or_else(|| {
                            CommandError::new(
                                "--amount is required",
                                "MISSING_ARG",
                                "Provide --amount=<sats>",
                            )
                        })?
                        .parse()
                        .map_err(|_| {
                            CommandError::new(
                                "--amount must be a number",
                                "INVALID_ARGS",
                                "Provide amount in satoshis",
                            )
                        })?;

                    let config = resolve_config();
                    let (mut db, mut wallet) = bdk_lib::load_wallet(&config).map_err(err)?;

                    let result = bdk_lib::send(
                        &mut wallet,
                        &mut db,
                        &config.esplora_url,
                        &to,
                        amount_sats,
                    )
                    .await
                    .map_err(err)?;

                    Ok(CommandOutput::new(json!(result)).next_actions(vec![
                        NextAction::new("bdk-agcli sync", "Sync to see updated balance"),
                        NextAction::new("bdk-agcli list-tx", "View transactions"),
                    ]))
                })
            },
        )
}

fn list_tx_command() -> Command {
    Command::new("list-tx", "List wallet transactions")
        .usage("bdk-agcli list-tx")
        .sync_handler(
            |_req: &CommandRequest<'_>, _ctx: &mut ExecutionContext| {
                let config = resolve_config();
                let (_db, wallet) = bdk_lib::load_wallet(&config).map_err(err)?;
                let txs = bdk_lib::list_transactions(&wallet);

                Ok(CommandOutput::new(json!({
                    "count": txs.len(),
                    "transactions": txs,
                }))
                .next_actions(vec![
                    NextAction::new("bdk-agcli sync", "Sync for latest"),
                    NextAction::new("bdk-agcli balance", "View balance"),
                ]))
            },
        )
}

#[tokio::main]
async fn main() {
    std::panic::set_hook(Box::new(|info| {
        let message = if let Some(msg) = info.payload().downcast_ref::<&str>() {
            (*msg).to_string()
        } else if let Some(msg) = info.payload().downcast_ref::<String>() {
            msg.clone()
        } else {
            "Unknown panic".to_string()
        };
        let envelope = ErrorEnvelope::new(
            "unknown",
            message,
            "INTERNAL_ERROR",
            "This is a bug — please report it",
            vec![],
        );
        let json = serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| {
            r#"{"ok":false,"error":{"message":"panic","code":"INTERNAL_ERROR"}}"#.to_string()
        });
        println!("{json}");
    }));

    let config = resolve_config();
    let wallet_exists = config.mnemonic_path().exists();

    let cli = AgentCli::new("bdk-agcli", "Agent CLI for Bitcoin wallet operations (BDK)")
        .version(env!("CARGO_PKG_VERSION"))
        .schema_version("bdk-agcli.v1")
        .root_field("wallet_initialized", json!(wallet_exists))
        .root_field("network", json!(config.network.to_string()))
        .command(init_command())
        .command(address_command())
        .command(sync_command())
        .command(balance_command())
        .command(send_command())
        .command(list_tx_command());

    let execution = cli.run_env().await;
    println!("{}", execution.to_json_pretty());
    process::exit(execution.exit_code());
}
