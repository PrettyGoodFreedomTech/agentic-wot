use dioxus::prelude::*;

use crate::routes::Route;

/// Displays a grid of list headers from the marketplace.
#[component]
pub fn ListBrowse() -> Element {
    // TODO: Wire up to NostrService to fetch real headers
    rsx! {
        div { class: "list-grid",
            p { class: "placeholder",
                "Connect to a relay to browse lists."
            }
        }
    }
}

/// A single list card for the browse view.
#[component]
pub fn ListCard(
    coordinate: String,
    name: String,
    description: Option<String>,
    item_count: usize,
    zap_count: u64,
) -> Element {
    rsx! {
        div { class: "list-card",
            Link { to: Route::ListDetailPage { coordinate: coordinate.clone() },
                h3 { "{name}" }
                if let Some(desc) = &description {
                    p { class: "description", "{desc}" }
                }
                div { class: "card-meta",
                    span { "{item_count} items" }
                    span { "{zap_count} zaps" }
                }
            }
        }
    }
}
