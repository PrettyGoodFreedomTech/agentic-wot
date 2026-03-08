use dioxus::prelude::*;

#[component]
pub fn ZapButton(coordinate: String) -> Element {
    rsx! {
        button {
            class: "inline-flex items-center gap-2 px-6 py-3 bg-amber-400 hover:bg-amber-300 text-gray-950 font-bold rounded-lg transition-colors cursor-pointer",
            onclick: move |_| {
                tracing::info!("Zap clicked for {}", coordinate);
            },
            i { class: "ph ph-lightning text-xl" }
            "Zap Curator"
        }
    }
}
