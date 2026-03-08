use dioxus::prelude::*;

use crate::routes::Route;
use crate::state::marketplace::MarketplaceState;
use crate::state::nostr::NostrState;
use crate::state::wallet::WalletState;

pub fn App() -> Element {
    // Provide global state via context
    use_context_provider(|| Signal::new(NostrState::default()));
    use_context_provider(|| Signal::new(WalletState::default()));
    use_context_provider(|| Signal::new(MarketplaceState::default()));

    rsx! {
        Router::<Route> {}
    }
}
