use std::collections::HashSet;
use std::time::Duration;

use nostr_sdk::prelude::*;

use crate::error::NostrLibError;
use crate::filters;
use crate::types::{MarketplaceList, kinds};

const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
const FETCH_PAGE_SIZE: usize = 500;

/// Commands that the UI can send to the `NostrService`.
#[derive(Debug)]
pub enum NostrCommand {
    /// Connect to a relay URL.
    AddRelay(String),
    /// Disconnect from a relay URL.
    RemoveRelay(String),
    /// Publish a header event.
    PublishHeader {
        params: dcosl_core::header::HeaderParams,
        addressable: bool,
    },
    /// Publish an item event.
    PublishItem {
        parent_z_ref: String,
        resource: String,
        fields: Vec<String>,
        content: Option<String>,
    },
}

/// Marketplace-aware Nostr client wrapping `nostr_sdk::Client`.
pub struct NostrService {
    client: Client,
}

impl NostrService {
    pub fn new(keys: Keys) -> Self {
        let client = Client::builder().signer(keys).build();
        Self { client }
    }

    pub fn new_readonly() -> Self {
        let client = Client::default();
        Self { client }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub async fn add_relay(&self, url: &str) -> Result<(), NostrLibError> {
        self.client
            .add_relay(url)
            .await
            .map_err(|_| NostrLibError::RelayConnection {
                url: url.to_string(),
            })?;
        Ok(())
    }

    pub async fn connect(&self) {
        self.client.connect().await;
    }

    pub async fn disconnect(&self) {
        self.client.disconnect().await;
    }

    /// Fetch all list headers, optionally filtered.
    pub async fn fetch_headers(
        &self,
        author: Option<PublicKey>,
        hashtag: Option<&str>,
    ) -> Result<Vec<Event>, NostrLibError> {
        let filter = filters::list_headers_filter(author, hashtag);
        self.fetch_all_events(filter).await
    }

    /// Fetch items for a list by z-ref.
    pub async fn fetch_items(&self, z_ref: &str) -> Result<Vec<Event>, NostrLibError> {
        let filter = filters::list_items_filter(z_ref);
        let events = self
            .client
            .fetch_events(filter, FETCH_TIMEOUT)
            .await
            .map_err(|e| NostrLibError::Sdk(e.to_string()))?;

        Ok(events.into_iter().collect())
    }

    /// Fetch a full `MarketplaceList` by coordinate.
    pub async fn fetch_marketplace_list(
        &self,
        coordinate: &str,
    ) -> Result<MarketplaceList, NostrLibError> {
        let (kind_num, pubkey, d_tag) = dcosl_core::item::parse_coordinate_str(coordinate)?;

        // Fetch header
        let header_filter = Filter::new()
            .kind(Kind::Custom(kind_num))
            .author(pubkey)
            .custom_tag(SingleLetterTag::lowercase(Alphabet::D), d_tag.clone())
            .limit(1);

        let header_events = self
            .client
            .fetch_events(header_filter, FETCH_TIMEOUT)
            .await
            .map_err(|e| NostrLibError::Sdk(e.to_string()))?;

        let header = header_events.into_iter().next().ok_or_else(|| {
            NostrLibError::Sdk(format!("Header not found for coordinate: {coordinate}"))
        })?;

        // Fetch items
        let items = self.fetch_items(coordinate).await?;

        // Extract metadata from header tags
        let header_json = dcosl_core::query::event_to_json(&header);
        let name = header_json["name"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        let plural_name = header_json["plural_name"].as_str().map(String::from);
        let description = header_json["description"].as_str().map(String::from);

        let categories: Vec<String> = header
            .tags
            .iter()
            .filter_map(|t| {
                let parts = t.as_slice();
                if parts.first().map(String::as_str) == Some("t") {
                    parts.get(1).cloned()
                } else {
                    None
                }
            })
            .collect();

        // Try to fetch curator profile — warn on failure so bugs are visible.
        let curator_profile = match crate::profile::fetch_profile(&self.client, pubkey).await {
            Ok(p) => Some(p),
            Err(e) => {
                tracing::warn!(
                    pubkey = %pubkey,
                    error = %e,
                    "failed to fetch curator profile for marketplace list"
                );
                None
            }
        };

        Ok(MarketplaceList {
            header,
            items,
            coordinate: coordinate.to_string(),
            name,
            plural_name,
            description,
            categories,
            zap_count: 0, // TODO: count zap receipts
            curator_pubkey: pubkey,
            curator_profile,
        })
    }

    /// Publish a header event and return its ID.
    ///
    /// Returns the `EventId` directly from the relay acknowledgement rather
    /// than re-fetching the event, which avoids false failures caused by
    /// eventual consistency on slow relays.
    pub async fn publish_header(
        &self,
        params: dcosl_core::header::HeaderParams,
        addressable: bool,
    ) -> Result<EventId, NostrLibError> {
        let kind = if addressable {
            kinds::HEADER
        } else {
            kinds::HEADER_REGULAR
        };

        let tags = dcosl_core::header::build_header_tags(&params);
        let builder = EventBuilder::new(kind, "").tags(tags);

        let output = self.client.send_event_builder(builder).await.map_err(|e| {
            NostrLibError::PublishFailed {
                reason: e.to_string(),
            }
        })?;

        Ok(output.val)
    }

    /// Publish an item event and return its ID.
    pub async fn publish_item(
        &self,
        parent_z_ref: &str,
        resource: &str,
        fields: &[String],
        content: Option<&str>,
        d_tag: Option<&str>,
    ) -> Result<EventId, NostrLibError> {
        let tags = dcosl_core::item::build_item_tags(
            parent_z_ref,
            resource,
            fields,
            d_tag,
            Some("magic-carpet"),
        );

        let builder = EventBuilder::new(kinds::ITEM, content.unwrap_or("")).tags(tags);

        let output = self.client.send_event_builder(builder).await.map_err(|e| {
            NostrLibError::PublishFailed {
                reason: e.to_string(),
            }
        })?;

        Ok(output.val)
    }

    /// Paginated fetch with dedupe.
    ///
    /// Uses the oldest timestamp in each batch as the next `until` value
    /// (without subtracting 1) to avoid dropping events that share a timestamp
    /// at a page boundary. Deduplication via `seen_ids` prevents re-processing.
    /// The loop terminates when a batch yields no new (unseen) events.
    async fn fetch_all_events(&self, base_filter: Filter) -> Result<Vec<Event>, NostrLibError> {
        const MAX_PAGES: usize = 100;

        let mut all_events: Vec<Event> = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();
        let mut until_secs: Option<u64> = None;

        for _ in 0..MAX_PAGES {
            let mut filter = base_filter.clone().limit(FETCH_PAGE_SIZE);
            if let Some(secs) = until_secs {
                filter = filter.until(Timestamp::from_secs(secs));
            }

            let batch = self
                .client
                .fetch_events(filter, FETCH_TIMEOUT)
                .await
                .map_err(|e| NostrLibError::Sdk(e.to_string()))?;

            if batch.is_empty() {
                break;
            }

            let mut new_in_batch = 0usize;
            let mut oldest_created_at = u64::MAX;
            for event in batch.iter() {
                oldest_created_at = oldest_created_at.min(event.created_at.as_secs());
                if seen_ids.insert(event.id.to_hex()) {
                    all_events.push(event.clone());
                    new_in_batch += 1;
                }
            }

            // No new events means we've exhausted this timestamp range.
            if new_in_batch == 0 || batch.len() < FETCH_PAGE_SIZE || oldest_created_at == 0 {
                break;
            }

            until_secs = Some(oldest_created_at);
        }

        Ok(all_events)
    }
}
