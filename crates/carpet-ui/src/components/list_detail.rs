use dioxus::prelude::*;

use crate::components::zap_button::ZapButton;
use crate::mock_data;
use crate::types::ListDisplay;

/// Detail view for a single list (header + items + zap button).
#[component]
pub fn ListDetail(coordinate: String) -> Element {
    let lists: Signal<Vec<ListDisplay>> = use_context();
    let list = lists.read().iter().find(|l| l.coordinate == coordinate).cloned();
    let items = mock_data::mock_items();

    match list {
        Some(list) => {
            let initial = list
                .curator_name
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string();

            let npub_display = list
                .curator_nip05
                .clone()
                .unwrap_or_else(|| "npub1...".into());

            rsx! {
                div { class: "max-w-4xl mx-auto px-4 py-8",
                    // Header
                    div { class: "mb-8",
                        div { class: "flex flex-wrap gap-2 mb-3",
                            for cat in list.categories.iter() {
                                span { class: "px-3 py-1 text-xs rounded-full bg-gray-800 text-gray-400",
                                    "{cat}"
                                }
                            }
                        }
                        h1 { class: "text-3xl font-bold text-gray-100 mb-2", "{list.name}" }
                        p { class: "text-gray-400 text-lg", "{list.description}" }
                    }

                    // Curator bar + zap button
                    div { class: "flex items-center justify-between bg-gray-900 border border-gray-800 rounded-xl p-4 mb-8",
                        div { class: "flex items-center gap-3",
                            div { class: "w-10 h-10 rounded-full bg-gray-800 flex items-center justify-center text-sm font-medium text-gray-300",
                                "{initial}"
                            }
                            div {
                                p { class: "text-gray-100 font-medium", "{list.curator_name}" }
                                p { class: "text-gray-500 text-sm font-mono", "{npub_display}" }
                            }
                        }
                        ZapButton { coordinate: coordinate.clone() }
                    }

                    // Stats
                    div { class: "flex items-center gap-6 mb-6 text-sm text-gray-400",
                        span { class: "flex items-center gap-1.5",
                            i { class: "ph ph-list-bullets" }
                            "{list.item_count} items"
                        }
                        span { class: "flex items-center gap-1.5 text-amber-400/70",
                            i { class: "ph ph-lightning" }
                            "{list.zap_count} zaps"
                        }
                    }

                    // Items
                    div {
                        h2 { class: "text-xl font-semibold text-gray-100 mb-4", "Items" }
                        div { class: "space-y-1",
                            for (i, item) in items.iter().enumerate() {
                                div {
                                    class: if i % 2 == 0 { "bg-gray-900 rounded-lg p-4" } else { "bg-gray-900/50 rounded-lg p-4" },
                                    div { class: "flex items-start justify-between mb-2",
                                        a {
                                            href: "{item.resource}",
                                            target: "_blank",
                                            class: "text-orange-500 hover:text-orange-400 text-sm font-mono break-all",
                                            "{item.resource}"
                                        }
                                    }
                                    p { class: "text-gray-300 text-sm mb-3", "{item.content}" }
                                    div { class: "flex flex-wrap gap-x-4 gap-y-1 text-xs text-gray-500",
                                        for (key, val) in item.fields.iter() {
                                            span {
                                                span { class: "text-gray-600", "{key}: " }
                                                span { class: "text-gray-400", "{val}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None => {
            rsx! {
                div { class: "max-w-4xl mx-auto px-4 py-8",
                    div { class: "text-center py-20",
                        i { class: "ph ph-magnifying-glass text-4xl text-gray-600 mb-4" }
                        h2 { class: "text-xl text-gray-400", "List not found" }
                        p { class: "text-gray-500 mt-2 font-mono text-sm", "{coordinate}" }
                    }
                }
            }
        }
    }
}
