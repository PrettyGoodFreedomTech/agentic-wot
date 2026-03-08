use dioxus::prelude::*;

use crate::components::layout::MainLayout;
use crate::components::list_card::ListBrowse;
use crate::components::list_detail::ListDetail;

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
        div { class: "marketplace",
            h1 { "Magic Carpet" }
            p { "Decentralized List Marketplace" }
            div { class: "featured-lists",
                h2 { "Featured Lists" }
                ListBrowse {}
            }
        }
    }
}

#[component]
fn ListBrowsePage() -> Element {
    rsx! {
        div { class: "list-browse",
            h1 { "Browse Lists" }
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
    rsx! {
        div { class: "bounty-browse",
            h1 { "Active Bounties" }
            p { "Coming soon — bounty listings will appear here." }
        }
    }
}

#[component]
fn WalletOverview() -> Element {
    rsx! {
        div { class: "wallet",
            h1 { "Wallet" }
            div { class: "balances",
                h2 { "Bitcoin (on-chain)" }
                p { "Balance: loading..." }
                h2 { "Lightning" }
                p { "Balance: loading..." }
            }
        }
    }
}

#[component]
fn Profile() -> Element {
    rsx! {
        div { class: "profile",
            h1 { "Profile" }
            p { "Key management and relay settings." }
        }
    }
}
