use dioxus::prelude::*;

/// Detail view for a single list (header + items + zap button).
#[component]
pub fn ListDetail(coordinate: String) -> Element {
    // TODO: Fetch full MarketplaceList via NostrService
    rsx! {
        div { class: "list-detail",
            h1 { "List: {coordinate}" }
            p { "Loading list details..." }

            div { class: "zap-section",
                button { class: "zap-button",
                    onclick: move |_| {
                        // TODO: Trigger NIP-57 zap flow
                        tracing::info!("Zap clicked for {}", coordinate);
                    },
                    "Zap Curator"
                }
            }

            div { class: "items-section",
                h2 { "Items" }
                p { "Loading items..." }
            }
        }
    }
}
