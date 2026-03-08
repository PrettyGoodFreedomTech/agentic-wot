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
        div { class: "min-h-screen bg-gray-950 text-gray-100",
            nav { class: "fixed top-0 left-0 right-0 z-50 bg-gray-900/80 backdrop-blur-md border-b border-gray-800",
                div { class: "max-w-7xl mx-auto px-4 h-14 flex items-center justify-between",
                    // Brand
                    Link {
                        to: Route::Marketplace {},
                        class: "flex items-center gap-2 text-orange-500 font-bold text-lg hover:text-orange-400 transition-colors",
                        i { class: "ph ph-flying-saucer text-2xl" }
                        "Magic Carpet"
                    }

                    // Nav links
                    div { class: "flex items-center gap-1",
                        NavLink { to: Route::Marketplace {}, icon: "ph-house", label: "Home" }
                        NavLink { to: Route::ListBrowsePage {}, icon: "ph-list-bullets", label: "Lists" }
                        NavLink { to: Route::BountyBrowse {}, icon: "ph-hand-coins", label: "Bounties" }
                        NavLink { to: Route::WalletOverview {}, icon: "ph-wallet", label: "Wallet" }
                        NavLink { to: Route::Profile {}, icon: "ph-user-circle", label: "Profile" }
                    }

                    // Status
                    div { class: "flex items-center gap-4",
                        span { class: "flex items-center gap-1.5 text-sm text-gray-400",
                            span { class: "w-2 h-2 rounded-full bg-emerald-400" }
                            "{relay_count} relays"
                        }
                        span { class: "flex items-center gap-1 text-amber-400 text-sm font-mono font-medium",
                            i { class: "ph ph-lightning" }
                            "{ln_balance} sats"
                        }
                    }
                }
            }

            main { class: "pt-14",
                Outlet::<Route> {}
            }
        }
    }
}

#[component]
fn NavLink(to: Route, icon: &'static str, label: &'static str) -> Element {
    rsx! {
        Link {
            to,
            class: "flex items-center gap-1.5 px-3 py-1.5 text-sm text-gray-400 hover:text-gray-200 hover:bg-gray-800 rounded-lg transition-colors",
            i { class: "ph {icon}" }
            "{label}"
        }
    }
}
