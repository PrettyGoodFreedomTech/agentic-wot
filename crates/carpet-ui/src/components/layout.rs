use dioxus::prelude::*;

use crate::routes::Route;
use crate::state::nostr::NostrState;
use crate::state::wallet::WalletState;

#[component]
pub fn MainLayout() -> Element {
    let nostr_state: Signal<NostrState> = use_context();
    let wallet_state: Signal<WalletState> = use_context();

    let relay_count = nostr_state.read().connected_relays.len();
    let ln_balance = wallet_state.read().ln_balance_sats;

    rsx! {
        div { class: "app-container",
            nav { class: "main-nav",
                div { class: "nav-brand",
                    Link { to: Route::Marketplace {}, "Magic Carpet" }
                }
                div { class: "nav-links",
                    Link { to: Route::Marketplace {}, "Home" }
                    Link { to: Route::ListBrowsePage {}, "Lists" }
                    Link { to: Route::BountyBrowse {}, "Bounties" }
                    Link { to: Route::WalletOverview {}, "Wallet" }
                    Link { to: Route::Profile {}, "Profile" }
                }
                div { class: "nav-status",
                    span { class: "relay-indicator",
                        "⚡ {relay_count} relays"
                    }
                    span { class: "wallet-bar",
                        "{ln_balance} sats"
                    }
                }
            }
            main { class: "main-content",
                Outlet::<Route> {}
            }
        }
    }
}
