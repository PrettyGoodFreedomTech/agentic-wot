/// Nostr connection state tracked in the UI.
#[derive(Debug, Clone, Default)]
pub struct NostrState {
    pub connected_relays: Vec<String>,
    pub is_connecting: bool,
    pub has_signer: bool,
    pub npub: Option<String>,
}
