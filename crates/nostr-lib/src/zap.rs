use nostr_sdk::prelude::*;

use crate::error::NostrLibError;
use crate::profile;

/// Construct a NIP-57 zap request (kind 9734).
///
/// This returns the zap request event — it should be sent to the LNURL callback,
/// NOT published to relays.
pub fn build_zap_request(
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

    if relay_urls.is_empty() {
        return Err(NostrLibError::Zap {
            reason: "no valid relay URLs for zap routing".into(),
        });
    }

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

    if user.is_empty() || domain.is_empty() || domain.contains('@') {
        return Err(NostrLibError::Lnurl {
            reason: format!("Invalid lud16 format: {lud16}"),
        });
    }

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

    // ── lud16 ───────────────────────────────────────────────────────

    #[test]
    fn lud16_to_lnurl_valid() {
        let url = lud16_to_lnurl_callback("user@example.com").unwrap();
        assert_eq!(url, "https://example.com/.well-known/lnurlp/user");
    }

    #[test]
    fn lud16_to_lnurl_invalid_no_at() {
        assert!(lud16_to_lnurl_callback("no-at-sign").is_err());
    }

    #[test]
    fn lud16_to_lnurl_empty_user() {
        assert!(lud16_to_lnurl_callback("@example.com").is_err());
    }

    #[test]
    fn lud16_to_lnurl_empty_domain() {
        assert!(lud16_to_lnurl_callback("user@").is_err());
    }

    #[test]
    fn lud16_to_lnurl_multiple_at() {
        assert!(lud16_to_lnurl_callback("user@a@b").is_err());
    }

    // ── build_zap_request ───────────────────────────────────────────

    fn test_keys() -> Keys {
        Keys::generate()
    }

    fn test_recipient() -> PublicKey {
        Keys::generate().public_key()
    }

    #[test]
    fn zap_request_valid_minimal() {
        let keys = test_keys();
        let recipient = test_recipient();
        let relays = vec!["wss://relay.example.com".to_string()];

        let event = build_zap_request(&keys, recipient, None, None, 1000, relays, None).unwrap();

        assert_eq!(event.kind, Kind::Custom(9734));
    }

    #[test]
    fn zap_request_with_event_id() {
        let keys = test_keys();
        let recipient = test_recipient();
        let relays = vec!["wss://relay.example.com".to_string()];
        let event_id = EventId::all_zeros();

        let event =
            build_zap_request(&keys, recipient, Some(event_id), None, 1000, relays, None).unwrap();

        let has_e_tag = event.tags.iter().any(|t| {
            let parts = t.as_slice();
            parts.first().map(String::as_str) == Some("e")
        });
        assert!(has_e_tag, "zap request should contain an 'e' tag");
    }

    #[test]
    fn zap_request_empty_relays_errors() {
        let keys = test_keys();
        let recipient = test_recipient();

        let result = build_zap_request(&keys, recipient, None, None, 1000, vec![], None);

        assert!(result.is_err());
    }

    #[test]
    fn zap_request_all_invalid_relays_errors() {
        let keys = test_keys();
        let recipient = test_recipient();
        let relays = vec!["not-a-url".to_string(), "also-bad".to_string()];

        let result = build_zap_request(&keys, recipient, None, None, 1000, relays, None);

        assert!(result.is_err());
    }

    #[test]
    fn zap_request_mixed_relays_keeps_valid() {
        let keys = test_keys();
        let recipient = test_recipient();
        let relays = vec!["not-a-url".to_string(), "wss://good.relay.com".to_string()];

        let result = build_zap_request(&keys, recipient, None, None, 1000, relays, None);

        assert!(result.is_ok());
    }
}
