use dioxus::prelude::*;

use crate::routes::Route;
use crate::types::ListDisplay;

/// Displays a grid of list cards from context.
#[component]
pub fn ListBrowse() -> Element {
    let lists: Signal<Vec<ListDisplay>> = use_context();

    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
            for list in lists.read().iter() {
                ListCard { key: "{list.coordinate}", list: list.clone() }
            }
        }
    }
}

/// A single list card for the browse view.
#[component]
pub fn ListCard(list: ListDisplay) -> Element {
    let initial = list
        .curator_name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    rsx! {
        Link {
            to: Route::ListDetailPage { coordinate: list.coordinate.clone() },
            class: "block bg-gray-900 border border-gray-800 rounded-xl p-5 hover:border-gray-700 hover:bg-gray-900/80 transition-colors group",

            // Category pills
            div { class: "flex flex-wrap gap-1.5 mb-3",
                for cat in list.categories.iter() {
                    span { class: "px-2 py-0.5 text-xs rounded-full bg-gray-800 text-gray-400",
                        "{cat}"
                    }
                }
            }

            // Title + description
            h3 { class: "text-gray-100 font-semibold text-lg mb-1 group-hover:text-orange-400 transition-colors",
                "{list.name}"
            }
            p { class: "text-gray-400 text-sm mb-4 line-clamp-2",
                "{list.description}"
            }

            // Footer: curator + stats
            div { class: "flex items-center justify-between",
                // Curator
                div { class: "flex items-center gap-2",
                    div { class: "w-7 h-7 rounded-full bg-gray-800 flex items-center justify-center text-xs font-medium text-gray-300",
                        "{initial}"
                    }
                    span { class: "text-gray-500 text-sm", "{list.curator_name}" }
                }
                // Stats
                div { class: "flex items-center gap-3 text-sm text-gray-500",
                    span { class: "flex items-center gap-1",
                        i { class: "ph ph-list-bullets" }
                        "{list.item_count}"
                    }
                    span { class: "flex items-center gap-1 text-amber-400/70",
                        i { class: "ph ph-lightning" }
                        "{list.zap_count}"
                    }
                }
            }
        }
    }
}
