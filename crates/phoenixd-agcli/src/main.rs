use std::process;

use agcli::{
    AgentCli, Command, CommandError, CommandOutput, CommandRequest, ErrorEnvelope,
    ExecutionContext, NextAction,
};
use phoenixd_lib::PhoenixdError;
use serde_json::json;

fn err(e: PhoenixdError) -> CommandError {
    let (code, retryable) = match &e {
        PhoenixdError::Http(_) => ("HTTP_ERROR", true),
        PhoenixdError::Api { .. } => ("API_ERROR", false),
        PhoenixdError::Deserialize(_) => ("DESERIALIZE_ERROR", false),
    };
    CommandError::new(
        e.to_string(),
        code,
        "Check PhoenixD is running and PHOENIXD_PASSWORD is set",
    )
    .retryable(retryable)
}

fn build_client() -> Result<phoenixd_lib::PhoenixdClient, CommandError> {
    phoenixd_lib::PhoenixdClient::from_env().map_err(err)
}

fn info_command() -> Command {
    Command::new("info", "Show PhoenixD node info")
        .usage("phoenixd-agcli info")
        .handler(
            |_req: &CommandRequest<'_>, _ctx: &mut ExecutionContext| {
                Box::pin(async move {
                    let client = build_client()?;
                    let info = client.node_info().await.map_err(err)?;
                    Ok(CommandOutput::new(json!(info)).next_actions(vec![
                        NextAction::new("phoenixd-agcli balance", "Check balance"),
                        NextAction::new("phoenixd-agcli channels", "List channels"),
                    ]))
                })
            },
        )
}

fn balance_command() -> Command {
    Command::new("balance", "Show wallet balance")
        .usage("phoenixd-agcli balance")
        .handler(
            |_req: &CommandRequest<'_>, _ctx: &mut ExecutionContext| {
                Box::pin(async move {
                    let client = build_client()?;
                    let balance = client.get_balance().await.map_err(err)?;
                    Ok(CommandOutput::new(json!(balance)).next_actions(vec![
                        NextAction::new(
                            "phoenixd-agcli invoice --amount <sats> --description <desc>",
                            "Create an invoice to receive",
                        ),
                        NextAction::new(
                            "phoenixd-agcli pay --invoice <bolt11>",
                            "Pay an invoice",
                        ),
                    ]))
                })
            },
        )
}

fn invoice_command() -> Command {
    Command::new("invoice", "Create a BOLT11 invoice")
        .usage("phoenixd-agcli invoice --amount <sats> --description <desc>")
        .handler(
            |req: &CommandRequest<'_>, _ctx: &mut ExecutionContext| {
                let amount_str = req.flag("amount").map(String::from);
                let description = req
                    .flag("description")
                    .unwrap_or("magic-carpet invoice")
                    .to_string();
                let external_id = req.flag("external-id").map(String::from);

                Box::pin(async move {
                    let amount: u64 = amount_str
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

                    let client = build_client()?;
                    let invoice = client
                        .create_invoice(amount, &description, external_id.as_deref())
                        .await
                        .map_err(err)?;

                    Ok(CommandOutput::new(json!(invoice)).next_actions(vec![
                        NextAction::new(
                            "phoenixd-agcli balance",
                            "Check balance after payment",
                        ),
                    ]))
                })
            },
        )
}

fn pay_command() -> Command {
    Command::new("pay", "Pay a BOLT11 invoice")
        .usage("phoenixd-agcli pay --invoice <bolt11>")
        .handler(
            |req: &CommandRequest<'_>, _ctx: &mut ExecutionContext| {
                let bolt11 = req.flag("invoice").map(String::from);

                Box::pin(async move {
                    let bolt11 = bolt11.ok_or_else(|| {
                        CommandError::new(
                            "--invoice is required",
                            "MISSING_ARG",
                            "Provide --invoice=<bolt11>",
                        )
                    })?;

                    let client = build_client()?;
                    let result = client.pay_invoice(&bolt11).await.map_err(err)?;

                    Ok(CommandOutput::new(json!(result)).next_actions(vec![
                        NextAction::new("phoenixd-agcli balance", "Check updated balance"),
                    ]))
                })
            },
        )
}

fn channels_command() -> Command {
    Command::new("channels", "List channels")
        .usage("phoenixd-agcli channels")
        .handler(
            |_req: &CommandRequest<'_>, _ctx: &mut ExecutionContext| {
                Box::pin(async move {
                    let client = build_client()?;
                    let channels = client.list_channels().await.map_err(err)?;
                    Ok(CommandOutput::new(json!({
                        "count": channels.len(),
                        "channels": channels,
                    }))
                    .next_actions(vec![
                        NextAction::new("phoenixd-agcli info", "Node info"),
                        NextAction::new("phoenixd-agcli balance", "Balance"),
                    ]))
                })
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

    let phoenixd_configured = std::env::var("PHOENIXD_PASSWORD").is_ok();

    let cli = AgentCli::new(
        "phoenixd-agcli",
        "Agent CLI for Lightning operations (PhoenixD)",
    )
    .version(env!("CARGO_PKG_VERSION"))
    .schema_version("phoenixd-agcli.v1")
    .root_field("phoenixd_configured", json!(phoenixd_configured))
    .command(info_command())
    .command(balance_command())
    .command(invoice_command())
    .command(pay_command())
    .command(channels_command());

    let execution = cli.run_env().await;
    println!("{}", execution.to_json_pretty());
    process::exit(execution.exit_code());
}
