use nostr_sdk::prelude::*;

use crate::error::NostrLibError;
use crate::profile;

/// Construct a NIP-57 zap request (kind 9734).
///
/// This returns the zap request event — it should be sent to the LNURL callback,
/// NOT published to relays.
#[allow(clippy::too_many_arguments)]
pub fn build_zap_request(
    _client: &Client,
    signer: &Keys,
    recipient_pubkey: PublicKey,
    target_event_id: Option<EventId>,
    target_coordinate: Option<String>,
    amount_msats: u64,
    relay_urls: Vec<String>,
    content: Option<String>,
) -> Result<Event, NostrLibError> {
    let relay_urls: Vec<RelayUrl> = relay_urls
        .into_iter()
        .filter_map(|u| RelayUrl::parse(&u).ok())
        .collect();

    let mut zap_data = ZapRequestData::new(recipient_pubkey, relay_urls)
        .amount(amount_msats)
        .message(content.unwrap_or_default());

    if let Some(event_id) = target_event_id {
        zap_data = zap_data.event_id(event_id);
    }
    if let Some(coord) = target_coordinate {
        // Parse coordinate and add as `a` tag
        let (kind_num, pubkey, d_tag) = dcosl_core::item::parse_coordinate_str(&coord)?;
        let coordinate = Coordinate::new(Kind::Custom(kind_num), pubkey).identifier(d_tag);
        zap_data = zap_data.event_coordinate(coordinate);
    }

    let zap_request = EventBuilder::public_zap_request(zap_data)
        .sign_with_keys(signer)
        .map_err(|e| NostrLibError::Zap {
            reason: e.to_string(),
        })?;

    Ok(zap_request)
}

/// Resolve a recipient's LNURL callback URL from their lud16.
///
/// lud16 format: `user@domain` → `https://domain/.well-known/lnurlp/user`
pub fn lud16_to_lnurl_callback(lud16: &str) -> Result<String, NostrLibError> {
    let (user, domain) = lud16.split_once('@').ok_or(NostrLibError::Lnurl {
        reason: format!("Invalid lud16 format: {lud16}"),
    })?;

    Ok(format!("https://{domain}/.well-known/lnurlp/{user}"))
}

/// Full NIP-57 zap flow:
/// 1. Fetch recipient's lud16 from kind 0 profile
/// 2. Build zap request event
/// 3. Return (`zap_request`, `lnurl_callback`) for the caller to send
pub async fn prepare_zap(
    client: &Client,
    signer: &Keys,
    recipient_pubkey: PublicKey,
    target_event_id: Option<EventId>,
    target_coordinate: Option<String>,
    amount_msats: u64,
    content: Option<String>,
) -> Result<(Event, String), NostrLibError> {
    let lud16 = profile::fetch_lud16(client, recipient_pubkey).await?;
    let callback_url = lud16_to_lnurl_callback(&lud16)?;

    // Use recipient's relay list for zap routing
    let relay_urls = profile::fetch_relay_list(client, recipient_pubkey).await?;
    let relay_urls = if relay_urls.is_empty() {
        // Fallback to connected relays
        client
            .relays()
            .await
            .keys()
            .map(std::string::ToString::to_string)
            .collect()
    } else {
        relay_urls
    };

    let zap_request = build_zap_request(
        client,
        signer,
        recipient_pubkey,
        target_event_id,
        target_coordinate,
        amount_msats,
        relay_urls,
        content,
    )?;

    Ok((zap_request, callback_url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lud16_to_lnurl_valid() {
        let url = lud16_to_lnurl_callback("user@example.com").unwrap();
        assert_eq!(url, "https://example.com/.well-known/lnurlp/user");
    }

    #[test]
    fn lud16_to_lnurl_invalid_format() {
        assert!(lud16_to_lnurl_callback("no-at-sign").is_err());
    }
}
