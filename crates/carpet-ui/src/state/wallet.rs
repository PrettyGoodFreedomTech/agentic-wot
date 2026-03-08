/// Wallet balances tracked in the UI.
#[derive(Debug, Clone, Default)]
pub struct WalletState {
    pub btc_balance_sats: u64,
    pub ln_balance_sats: u64,
    pub is_loading: bool,
}
