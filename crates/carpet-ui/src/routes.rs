use dioxus::prelude::*;

use crate::components::bounty_card::BountyCard;
use crate::components::layout::MainLayout;
use crate::components::list_card::ListBrowse;
use crate::components::list_detail::ListDetail;
use crate::components::wallet_card::{WalletCard, WalletVariant};
use crate::mock_data;
use crate::state::nostr::NostrState;
use crate::state::wallet::WalletState;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(MainLayout)]
    #[route("/")]
    Marketplace {},

    #[route("/lists")]
    ListBrowsePage {},

    #[route("/lists/:coordinate")]
    ListDetailPage { coordinate: String },

    #[route("/bounties")]
    BountyBrowse {},

    #[route("/wallet")]
    WalletOverview {},

    #[route("/profile")]
    Profile {},
}

// --- Page components ---

#[component]
fn Marketplace() -> Element {
    rsx! {
        div { class: "max-w-7xl mx-auto px-4 py-8",
            // Hero
            div { class: "text-center mb-10",
                h1 { class: "text-4xl font-bold text-gray-100 mb-3",
                    "Discover Curated Lists"
                }
                p { class: "text-gray-400 text-lg max-w-2xl mx-auto",
                    "A decentralized marketplace for community-curated lists, powered by Nostr and Lightning."
                }
            }

            // Search
            div { class: "max-w-xl mx-auto mb-8",
                div { class: "relative",
                    i { class: "ph ph-magnifying-glass absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" }
                    input {
                        r#type: "text",
                        placeholder: "Search lists...",
                        class: "w-full bg-gray-900 border border-gray-800 rounded-lg pl-10 pr-4 py-2.5 text-gray-200 placeholder-gray-500 focus:outline-none focus:border-gray-700",
                    }
                }
            }

            // Category filters
            div { class: "flex flex-wrap justify-center gap-2 mb-8",
                for cat in ["All", "Books", "Wallets", "Nostr", "Podcasts", "Privacy", "Development"] {
                    button {
                        class: if cat == "All" {
                            "px-4 py-1.5 text-sm rounded-full bg-orange-500 text-white font-medium cursor-pointer"
                        } else {
                            "px-4 py-1.5 text-sm rounded-full bg-gray-900 text-gray-400 hover:bg-gray-800 hover:text-gray-200 transition-colors border border-gray-800 cursor-pointer"
                        },
                        "{cat}"
                    }
                }
            }

            // Grid
            ListBrowse {}
        }
    }
}

#[component]
fn ListBrowsePage() -> Element {
    let lists: Signal<Vec<crate::types::ListDisplay>> = use_context();
    let count = lists.read().len();

    rsx! {
        div { class: "max-w-7xl mx-auto px-4 py-8",
            div { class: "flex items-center justify-between mb-6",
                div { class: "flex items-center gap-3",
                    h1 { class: "text-2xl font-bold text-gray-100", "Browse Lists" }
                    span { class: "px-2.5 py-0.5 text-xs rounded-full bg-gray-800 text-gray-400 font-mono",
                        "{count}"
                    }
                }
                div { class: "relative",
                    i { class: "ph ph-magnifying-glass absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" }
                    input {
                        r#type: "text",
                        placeholder: "Search...",
                        class: "bg-gray-900 border border-gray-800 rounded-lg pl-10 pr-4 py-2 text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:border-gray-700",
                    }
                }
            }
            ListBrowse {}
        }
    }
}

#[component]
fn ListDetailPage(coordinate: String) -> Element {
    rsx! {
        ListDetail { coordinate }
    }
}

#[component]
fn BountyBrowse() -> Element {
    let bounties = mock_data::mock_bounties();

    rsx! {
        div { class: "max-w-7xl mx-auto px-4 py-8",
            div { class: "flex items-center gap-3 mb-6",
                h1 { class: "text-2xl font-bold text-gray-100", "Active Bounties" }
                span { class: "px-2.5 py-0.5 text-xs rounded-full bg-gray-800 text-gray-400 font-mono",
                    "{bounties.len()}"
                }
            }
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                for bounty in bounties {
                    BountyCard { bounty }
                }
            }
        }
    }
}

#[component]
fn WalletOverview() -> Element {
    let wallet_state: Signal<WalletState> = use_context();
    let wallet = wallet_state.read();

    rsx! {
        div { class: "max-w-3xl mx-auto px-4 py-8",
            h1 { class: "text-2xl font-bold text-gray-100 mb-6", "Wallet" }
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                WalletCard { variant: WalletVariant::Bitcoin, balance_sats: wallet.btc_balance_sats }
                WalletCard { variant: WalletVariant::Lightning, balance_sats: wallet.ln_balance_sats }
            }
        }
    }
}

#[component]
fn Profile() -> Element {
    let nostr_state: Signal<NostrState> = use_context();
    let nostr = nostr_state.read();

    let npub_display = nostr
        .npub
        .as_ref()
        .map(|n| {
            if n.len() > 20 {
                format!("{}...{}", &n[..12], &n[n.len() - 8..])
            } else {
                n.clone()
            }
        })
        .unwrap_or_else(|| "Not connected".into());

    rsx! {
        div { class: "max-w-3xl mx-auto px-4 py-8",
            // Profile header
            div { class: "bg-gray-900 border border-gray-800 rounded-xl p-6 mb-6",
                div { class: "flex items-center gap-4 mb-4",
                    div { class: "w-16 h-16 rounded-full bg-gray-800 flex items-center justify-center text-2xl font-bold text-orange-500",
                        "M"
                    }
                    div {
                        h1 { class: "text-2xl font-bold text-gray-100", "Magic Carpet User" }
                        p { class: "text-gray-500 text-sm font-mono flex items-center gap-2",
                            "{npub_display}"
                            button { class: "text-gray-600 hover:text-gray-400 transition-colors cursor-pointer",
                                i { class: "ph ph-copy" }
                            }
                        }
                    }
                }
            }

            // Relays
            div { class: "bg-gray-900 border border-gray-800 rounded-xl p-6 mb-6",
                h2 { class: "text-lg font-semibold text-gray-100 mb-4 flex items-center gap-2",
                    i { class: "ph ph-globe text-gray-400" }
                    "Connected Relays"
                }
                div { class: "space-y-2",
                    for relay in nostr.connected_relays.iter() {
                        div { class: "flex items-center gap-2 text-sm",
                            span { class: "w-2 h-2 rounded-full bg-emerald-400" }
                            span { class: "text-gray-300 font-mono", "{relay}" }
                        }
                    }
                }
            }

            // Key management
            div { class: "bg-gray-900 border border-gray-800 rounded-xl p-6",
                h2 { class: "text-lg font-semibold text-gray-100 mb-4 flex items-center gap-2",
                    i { class: "ph ph-key text-gray-400" }
                    "Key Management"
                }
                p { class: "text-gray-400 text-sm",
                    "Key import, export, and NIP-46 remote signer configuration coming soon."
                }
            }
        }
    }
}
