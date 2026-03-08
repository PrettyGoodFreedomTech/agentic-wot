use nostr_sdk::prelude::*;

use crate::types::kinds;

/// Build a filter for DCoSL list headers (kinds 9998 + 39998).
pub fn list_headers_filter(author: Option<PublicKey>, hashtag: Option<&str>) -> Filter {
    let mut filter = Filter::new().kinds(vec![kinds::HEADER_REGULAR, kinds::HEADER]);

    if let Some(pk) = author {
        filter = filter.author(pk);
    }
    if let Some(t) = hashtag {
        filter = filter.hashtag(t);
    }

    filter
}

/// Build a filter for list items matching a parent z-ref.
pub fn list_items_filter(z_ref: &str) -> Filter {
    Filter::new()
        .kinds(vec![kinds::ITEM_REGULAR, kinds::ITEM])
        .custom_tag(SingleLetterTag::lowercase(Alphabet::Z), z_ref.to_string())
}

/// Build a filter for bounties (NIP-99 kind 30402) with bounty hashtag.
pub fn bounties_filter(target_coordinate: Option<&str>) -> Filter {
    let mut filter = Filter::new()
        .kind(kinds::BOUNTY)
        .hashtag("bounty");

    if let Some(coord) = target_coordinate {
        filter = filter.custom_tag(SingleLetterTag::lowercase(Alphabet::A), coord.to_string());
    }

    filter
}

/// Build a filter for zap receipts (kind 9735) targeting a specific event or coordinate.
pub fn zap_receipts_filter(target_pubkey: PublicKey) -> Filter {
    Filter::new()
        .kind(kinds::ZAP_RECEIPT)
        .pubkey(target_pubkey)
}

/// Build a filter for profile metadata (kind 0).
pub fn profile_filter(pubkey: PublicKey) -> Filter {
    Filter::new()
        .kind(Kind::Metadata)
        .author(pubkey)
        .limit(1)
}

/// Build a filter for relay list (NIP-65 kind 10002).
pub fn relay_list_filter(pubkey: PublicKey) -> Filter {
    Filter::new()
        .kind(kinds::RELAY_LIST)
        .author(pubkey)
        .limit(1)
}
