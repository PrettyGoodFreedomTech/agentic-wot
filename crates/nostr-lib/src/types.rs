use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// A marketplace list: header event + items + metadata.
#[derive(Debug, Clone)]
pub struct MarketplaceList {
    pub header: Event,
    pub items: Vec<Event>,
    pub coordinate: String,
    pub name: String,
    pub plural_name: Option<String>,
    pub description: Option<String>,
    pub categories: Vec<String>,
    pub zap_count: u64,
    pub curator_pubkey: PublicKey,
    pub curator_profile: Option<ProfileMetadata>,
}

/// Curator profile metadata (kind 0).
///
/// Uses a lenient deserializer for each field so that non-string values
/// (e.g. a numeric `name`) are coerced to `None` instead of failing the
/// entire struct. Kind 0 events in the wild vary significantly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    #[serde(default, deserialize_with = "lenient_string")]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "lenient_string")]
    pub about: Option<String>,
    #[serde(default, deserialize_with = "lenient_string")]
    pub picture: Option<String>,
    #[serde(default, deserialize_with = "lenient_string")]
    pub nip05: Option<String>,
    #[serde(default, deserialize_with = "lenient_string")]
    pub lud16: Option<String>,
    #[serde(default, deserialize_with = "lenient_string")]
    pub display_name: Option<String>,
}

/// Deserialize an `Option<String>` leniently: accept strings normally,
/// coerce non-string values (numbers, booleans, objects, etc.) to `None`.
fn lenient_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde_json::Value;
    let v = Option::<Value>::deserialize(deserializer)?;
    Ok(v.and_then(|val| match val {
        Value::String(s) => Some(s),
        _ => None,
    }))
}

/// A bounty targeting a specific list (NIP-99 kind 30402).
#[derive(Debug, Clone)]
pub struct Bounty {
    pub event: Event,
    pub d_tag: String,
    pub target_list_coordinate: String,
    pub reward_sats: u64,
    pub criteria: String,
    pub expiry: Option<Timestamp>,
    pub status: BountyStatus,
    pub creator_pubkey: PublicKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BountyStatus {
    Open,
    Fulfilled,
    Expired,
}

/// Links a list update event to a bounty it fulfills.
#[derive(Debug, Clone)]
pub struct BountyFulfillment {
    pub bounty_id: String,
    pub update_event_id: EventId,
    pub updater_pubkey: PublicKey,
    pub zap_receipt_id: Option<EventId>,
}

/// `DCoSL` event kinds used in the marketplace.
pub mod kinds {
    use nostr_sdk::Kind;

    pub const HEADER: Kind = Kind::Custom(39998);
    pub const HEADER_REGULAR: Kind = Kind::Custom(9998);
    pub const ITEM: Kind = Kind::Custom(39999);
    pub const ITEM_REGULAR: Kind = Kind::Custom(9999);
    pub const BOUNTY: Kind = Kind::Custom(30402);
    pub const ZAP_REQUEST: Kind = Kind::Custom(9734);
    pub const ZAP_RECEIPT: Kind = Kind::Custom(9735);
    pub const RELAY_LIST: Kind = Kind::Custom(10002);
}
