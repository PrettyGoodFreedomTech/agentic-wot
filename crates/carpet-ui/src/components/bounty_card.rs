use dioxus::prelude::*;

use crate::routes::Route;
use crate::types::{BountyDisplay, BountyStatus};

#[component]
pub fn BountyCard(bounty: BountyDisplay) -> Element {
    let (badge_class, badge_text) = match bounty.status {
        BountyStatus::Open => (
            "bg-emerald-400/10 text-emerald-400 border border-emerald-400/20",
            "Open",
        ),
        BountyStatus::Fulfilled => (
            "bg-blue-400/10 text-blue-400 border border-blue-400/20",
            "Fulfilled",
        ),
        BountyStatus::Expired => (
            "bg-gray-400/10 text-gray-400 border border-gray-400/20",
            "Expired",
        ),
    };

    rsx! {
        div { class: "bg-gray-900 border border-gray-800 rounded-xl p-5 hover:border-gray-700 transition-colors",
            div { class: "flex items-center justify-between mb-3",
                span { class: "px-3 py-1 rounded-full text-xs font-medium {badge_class}",
                    "{badge_text}"
                }
                span { class: "text-amber-400 font-mono font-bold text-lg",
                    i { class: "ph ph-lightning mr-1" }
                    "{bounty.reward_sats} sats"
                }
            }
            h3 { class: "text-gray-100 font-semibold mb-2",
                "Bounty for "
                Link {
                    to: Route::ListDetailPage { coordinate: bounty.target_list_coordinate },
                    class: "text-orange-500 hover:text-orange-400",
                    "{bounty.target_list_name}"
                }
            }
            p { class: "text-gray-400 text-sm mb-4",
                "{bounty.criteria}"
            }
            div { class: "flex items-center gap-2 text-gray-500 text-sm",
                i { class: "ph ph-user-circle" }
                span { "{bounty.creator_name}" }
            }
        }
    }
}
