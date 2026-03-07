use std::path::PathBuf;
use std::process::Command;

use bdk_lib::*;
use bdk_wallet::bitcoin::Network;
use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    std::env::var("WORKSPACE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf()
        })
}

fn bitcoin_cli(args: &[&str]) -> String {
    let compose_file = workspace_root().join("docker-compose.yml");
    let output = Command::new("docker")
        .args([
            "compose",
            "-f",
            compose_file.to_str().unwrap(),
            "exec",
            "-T",
            "bitcoind",
            "bitcoin-cli",
            "-regtest",
            "-rpcuser=user",
            "-rpcpassword=password",
            "-rpcwallet=default",
        ])
        .args(args)
        .output()
        .expect("failed to run docker compose exec");

    assert!(
        output.status.success(),
        "bitcoin-cli {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn make_config(tmp: &TempDir) -> WalletConfig {
    let esplora_url =
        std::env::var("BDK_ESPLORA_URL").unwrap_or_else(|_| "http://localhost:3002".to_string());
    WalletConfig {
        network: Network::Regtest,
        esplora_url,
        data_dir: tmp.path().to_path_buf(),
    }
}

async fn wait_for_sync(
    wallet: &mut wallet::BdkWallet,
    db: &mut wallet::WalletStore,
    esplora_url: &str,
    timeout_secs: u64,
) {
    let start = std::time::Instant::now();
    loop {
        sync_wallet(wallet, db, esplora_url).await.unwrap();
        let balance = get_balance(wallet);
        if balance.confirmed_sats > 0 {
            return;
        }
        if start.elapsed().as_secs() > timeout_secs {
            panic!(
                "Timed out after {}s waiting for confirmed balance",
                timeout_secs
            );
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

#[tokio::test]
#[ignore]
async fn test_full_wallet_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let config = make_config(&tmp);

    // Init wallet
    let creation = init_wallet(&config).unwrap();
    assert!(!creation.mnemonic.is_empty());

    // Load wallet
    let (mut db, mut wallet) = load_wallet(&config).unwrap();

    // Get address
    let addr = next_address(&mut wallet, &mut db).unwrap();
    assert!(addr.address.starts_with("bcrt1"), "got: {}", addr.address);

    // Fund wallet via bitcoind
    bitcoin_cli(&["sendtoaddress", &addr.address, "1.0"]);
    bitcoin_cli(&["-generate", "1"]);

    // Wait for sync and verify balance
    wait_for_sync(&mut wallet, &mut db, &config.esplora_url, 60).await;
    let balance = get_balance(&wallet);
    assert_eq!(balance.confirmed_sats, 100_000_000);

    // Get second address for self-send
    let addr2 = next_address(&mut wallet, &mut db).unwrap();

    // Send 50M sats to second address
    let send_result = send(&mut wallet, &mut db, &config.esplora_url, &addr2.address, 50_000_000)
        .await
        .unwrap();
    assert!(!send_result.txid.is_empty());
    assert!(send_result.fee_sats > 0);

    // Mine and sync again
    bitcoin_cli(&["-generate", "1"]);
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    sync_wallet(&mut wallet, &mut db, &config.esplora_url)
        .await
        .unwrap();

    // Verify transactions
    let txs = list_transactions(&wallet);
    assert!(txs.len() >= 2, "expected >= 2 txs, got {}", txs.len());

    // Balance should be ~50M minus fee (fee < 100k sats)
    let final_balance = get_balance(&wallet);
    assert!(
        (49_900_000..50_000_000).contains(&final_balance.confirmed_sats)
            || final_balance.confirmed_sats == 100_000_000 - send_result.fee_sats,
        "unexpected balance: {}",
        final_balance.confirmed_sats
    );
}

#[tokio::test]
#[ignore]
async fn test_wallet_already_exists() {
    let tmp = TempDir::new().unwrap();
    let config = make_config(&tmp);

    init_wallet(&config).unwrap();
    let err = init_wallet(&config).unwrap_err();
    assert!(
        matches!(err, BdkLibError::WalletAlreadyExists { .. }),
        "expected WalletAlreadyExists, got: {:?}",
        err
    );
}

#[tokio::test]
#[ignore]
async fn test_sync_empty_wallet() {
    let tmp = TempDir::new().unwrap();
    let config = make_config(&tmp);

    init_wallet(&config).unwrap();
    let (mut db, mut wallet) = load_wallet(&config).unwrap();

    sync_wallet(&mut wallet, &mut db, &config.esplora_url)
        .await
        .unwrap();

    let balance = get_balance(&wallet);
    assert_eq!(balance.total_sats, 0);
}
