use std::time::Duration;

use nostr_sdk::prelude::*;

use crate::error::NostrLibError;
use crate::filters;
use crate::types::ProfileMetadata;

const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// Fetch kind 0 profile metadata for a pubkey.
pub async fn fetch_profile(
    client: &Client,
    pubkey: PublicKey,
) -> Result<ProfileMetadata, NostrLibError> {
    let filter = filters::profile_filter(pubkey);
    let events = client
        .fetch_events(filter, FETCH_TIMEOUT)
        .await
        .map_err(|e| NostrLibError::Sdk(e.to_string()))?;

    let event = events
        .into_iter()
        .next()
        .ok_or(NostrLibError::ProfileNotFound {
            pubkey: pubkey.to_hex(),
        })?;

    serde_json::from_str(&event.content).map_err(|_| NostrLibError::ProfileNotFound {
        pubkey: pubkey.to_hex(),
    })
}

/// Extract lud16 (Lightning Address) from a profile.
pub async fn fetch_lud16(client: &Client, pubkey: PublicKey) -> Result<String, NostrLibError> {
    let profile = fetch_profile(client, pubkey).await?;
    profile.lud16.ok_or(NostrLibError::MissingLud16 {
        pubkey: pubkey.to_hex(),
    })
}

/// Fetch NIP-65 relay list (kind 10002) for a pubkey.
pub async fn fetch_relay_list(
    client: &Client,
    pubkey: PublicKey,
) -> Result<Vec<String>, NostrLibError> {
    let filter = filters::relay_list_filter(pubkey);
    let events = client
        .fetch_events(filter, FETCH_TIMEOUT)
        .await
        .map_err(|e| NostrLibError::Sdk(e.to_string()))?;

    let Some(event) = events.into_iter().next() else {
        return Ok(vec![]);
    };

    let relays: Vec<String> = event
        .tags
        .iter()
        .filter_map(|t| {
            let parts = t.as_slice();
            if parts.first().map(String::as_str) == Some("r") {
                parts.get(1).cloned()
            } else {
                None
            }
        })
        .collect();

    Ok(relays)
}
