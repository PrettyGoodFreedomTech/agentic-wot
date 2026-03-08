use dioxus::prelude::*;

use crate::mock_data;
use crate::routes::Route;
use crate::state::marketplace::MarketplaceState;
static TAILWIND: Asset = asset!("/assets/tailwind.css");
static PHOSPHOR: Asset = asset!("/assets/phosphor.css");

pub fn App() -> Element {
    use_context_provider(|| Signal::new(mock_data::nostr_state()));
    use_context_provider(|| Signal::new(mock_data::wallet_state()));
    use_context_provider(|| Signal::new(MarketplaceState::default()));
    use_context_provider(|| Signal::new(mock_data::mock_lists()));

    rsx! {
        document::Stylesheet { href: TAILWIND }
        document::Stylesheet { href: PHOSPHOR }
        Router::<Route> {}
    }
}
