use crate::state::nostr::NostrState;
use crate::state::wallet::WalletState;
use crate::types::{BountyDisplay, BountyStatus, ItemDisplay, ListDisplay};

pub fn mock_lists() -> Vec<ListDisplay> {
    vec![
        ListDisplay {
            coordinate: "30001:abc123:best-bitcoin-books".into(),
            name: "Best Bitcoin Books".into(),
            description: "Essential reading for understanding Bitcoin, from technical deep-dives to economic philosophy.".into(),
            categories: vec!["Books".into(), "Education".into()],
            item_count: 12,
            zap_count: 847,
            curator_name: "Satoshi Reader".into(),
            curator_picture: None,
            curator_nip05: Some("reader@bitcoinbooks.com".into()),
        },
        ListDisplay {
            coordinate: "30001:def456:lightning-wallets".into(),
            name: "Lightning Wallets".into(),
            description: "The best Lightning Network wallets ranked by usability, features, and sovereignty.".into(),
            categories: vec!["Wallets".into(), "Lightning".into()],
            item_count: 8,
            zap_count: 1203,
            curator_name: "LN Explorer".into(),
            curator_picture: None,
            curator_nip05: Some("explorer@ln.tips".into()),
        },
        ListDisplay {
            coordinate: "30001:ghi789:nostr-clients".into(),
            name: "Nostr Clients".into(),
            description: "A comprehensive guide to Nostr clients across all platforms — mobile, desktop, and web.".into(),
            categories: vec!["Nostr".into(), "Apps".into()],
            item_count: 15,
            zap_count: 2100,
            curator_name: "Nostr Enthusiast".into(),
            curator_picture: None,
            curator_nip05: Some("fan@nostr.com".into()),
        },
        ListDisplay {
            coordinate: "30001:jkl012:bitcoin-podcasts".into(),
            name: "Bitcoin Podcasts".into(),
            description: "Top podcasts covering Bitcoin technology, economics, and culture.".into(),
            categories: vec!["Podcasts".into(), "Media".into()],
            item_count: 10,
            zap_count: 523,
            curator_name: "Pod Collector".into(),
            curator_picture: None,
            curator_nip05: None,
        },
        ListDisplay {
            coordinate: "30001:mno345:privacy-tools".into(),
            name: "Privacy Tools".into(),
            description: "Tools and services for protecting your privacy in the digital age.".into(),
            categories: vec!["Privacy".into(), "Security".into()],
            item_count: 7,
            zap_count: 341,
            curator_name: "Privacy Max".into(),
            curator_picture: None,
            curator_nip05: Some("max@privacytools.io".into()),
        },
        ListDisplay {
            coordinate: "30001:pqr678:dev-resources".into(),
            name: "Bitcoin Dev Resources".into(),
            description: "Libraries, frameworks, and learning resources for Bitcoin developers.".into(),
            categories: vec!["Development".into(), "Education".into()],
            item_count: 20,
            zap_count: 1580,
            curator_name: "Dev Builder".into(),
            curator_picture: None,
            curator_nip05: Some("builder@devs.bitcoin".into()),
        },
    ]
}

pub fn mock_bounties() -> Vec<BountyDisplay> {
    vec![
        BountyDisplay {
            d_tag: "bounty-001".into(),
            target_list_name: "Best Bitcoin Books".into(),
            target_list_coordinate: "30001:abc123:best-bitcoin-books".into(),
            reward_sats: 50_000,
            criteria: "Add 3 books published in 2025 covering Bitcoin Layer 2 protocols".into(),
            status: BountyStatus::Open,
            creator_name: "Bookworm".into(),
        },
        BountyDisplay {
            d_tag: "bounty-002".into(),
            target_list_name: "Nostr Clients".into(),
            target_list_coordinate: "30001:ghi789:nostr-clients".into(),
            reward_sats: 25_000,
            criteria: "Review and rate all listed clients on NIP-44 support".into(),
            status: BountyStatus::Open,
            creator_name: "NIP Checker".into(),
        },
        BountyDisplay {
            d_tag: "bounty-003".into(),
            target_list_name: "Privacy Tools".into(),
            target_list_coordinate: "30001:mno345:privacy-tools".into(),
            reward_sats: 100_000,
            criteria: "Add Tor-based tools and verify each tool's open-source status".into(),
            status: BountyStatus::Fulfilled,
            creator_name: "Privacy Max".into(),
        },
    ]
}

pub fn mock_items() -> Vec<ItemDisplay> {
    vec![
        ItemDisplay {
            resource: "https://www.amazon.com/Bitcoin-Standard-Decentralized-Alternative-Central/dp/1119473861".into(),
            content: "The foundational text on Bitcoin's economic significance. Saifedean Ammous makes the case for Bitcoin as sound money.".into(),
            fields: vec![
                ("Author".into(), "Saifedean Ammous".into()),
                ("Year".into(), "2018".into()),
                ("Rating".into(), "5/5".into()),
            ],
        },
        ItemDisplay {
            resource: "https://www.amazon.com/Mastering-Bitcoin-Programming-Open-Blockchain/dp/1491954388".into(),
            content: "The technical bible for Bitcoin developers. Covers everything from keys to mining to the protocol.".into(),
            fields: vec![
                ("Author".into(), "Andreas M. Antonopoulos".into()),
                ("Year".into(), "2017".into()),
                ("Rating".into(), "5/5".into()),
            ],
        },
        ItemDisplay {
            resource: "https://www.amazon.com/Broken-Money-Financial-System-Failing/dp/B0CG8837J5".into(),
            content: "A deep exploration of monetary history and how Bitcoin fits into the evolution of money.".into(),
            fields: vec![
                ("Author".into(), "Lyn Alden".into()),
                ("Year".into(), "2023".into()),
                ("Rating".into(), "4.5/5".into()),
            ],
        },
        ItemDisplay {
            resource: "https://www.amazon.com/Sovereign-Individual-Mastering-Transition-Information/dp/0684832720".into(),
            content: "Prescient predictions about the impact of digital technology on society and economics.".into(),
            fields: vec![
                ("Author".into(), "James Dale Davidson".into()),
                ("Year".into(), "1999".into()),
                ("Rating".into(), "4/5".into()),
            ],
        },
    ]
}

pub fn nostr_state() -> NostrState {
    NostrState {
        connected_relays: vec![
            "wss://relay.damus.io".into(),
            "wss://nos.lol".into(),
            "wss://relay.nostr.band".into(),
        ],
        is_connecting: false,
        has_signer: true,
        npub: Some("npub1a2b3c4d5e6f7890abcdef1234567890abcdef1234567890abcdef12345678".into()),
    }
}

pub fn wallet_state() -> WalletState {
    WalletState {
        btc_balance_sats: 1_250_000,
        ln_balance_sats: 350_000,
        is_loading: false,
    }
}
