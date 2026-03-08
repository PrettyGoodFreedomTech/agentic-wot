use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum WalletVariant {
    Bitcoin,
    Lightning,
}

#[component]
pub fn WalletCard(variant: WalletVariant, balance_sats: u64) -> Element {
    let (icon, label, accent) = match variant {
        WalletVariant::Bitcoin => ("ph ph-currency-btc", "Bitcoin (on-chain)", "text-orange-500"),
        WalletVariant::Lightning => ("ph ph-lightning", "Lightning", "text-amber-400"),
    };

    let formatted = format_sats(balance_sats);

    rsx! {
        div { class: "bg-gray-900 border border-gray-800 rounded-xl p-6",
            div { class: "flex items-center gap-3 mb-4",
                div { class: "w-10 h-10 rounded-lg bg-gray-800 flex items-center justify-center",
                    i { class: "{icon} text-xl {accent}" }
                }
                span { class: "text-gray-400 text-sm font-medium", "{label}" }
            }
            div { class: "mb-4",
                span { class: "text-3xl font-bold font-mono text-gray-100", "{formatted}" }
                span { class: "text-gray-500 text-sm ml-2", "sats" }
            }
            div { class: "flex gap-3",
                button { class: "flex-1 py-2 px-4 bg-gray-800 hover:bg-gray-700 text-gray-200 rounded-lg text-sm font-medium transition-colors cursor-pointer",
                    "Receive"
                }
                button { class: "flex-1 py-2 px-4 bg-gray-800 hover:bg-gray-700 text-gray-200 rounded-lg text-sm font-medium transition-colors cursor-pointer",
                    "Send"
                }
            }
        }
    }
}

fn format_sats(sats: u64) -> String {
    let s = sats.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
