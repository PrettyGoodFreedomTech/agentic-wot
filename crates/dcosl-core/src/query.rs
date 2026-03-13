use nostr::prelude::*;
use serde_json::json;

/// Convert a nostr Event into a JSON representation with extracted DCoSL fields.
pub fn event_to_json(event: &Event) -> serde_json::Value {
    let tags: Vec<Vec<String>> = event
        .tags
        .iter()
        .map(|t| t.as_slice().iter().map(ToString::to_string).collect())
        .collect();

    let mut obj = json!({
        "event_id": event.id.to_hex(),
        "kind": event.kind.as_u16(),
        "pubkey": event.pubkey.to_hex(),
        "created_at": event.created_at.as_secs(),
        "tags": tags,
        "content": event.content,
        "sig": event.sig.to_string(),
    });

    // Extract common DCoSL fields from tags for convenience
    for tag in event.tags.iter() {
        let parts = tag.as_slice();
        if parts.len() >= 2 {
            let key = parts[0].as_str();
            match key {
                "names" => {
                    obj["name"] = json!(parts[1].as_str());
                    if parts.len() >= 3 {
                        obj["plural_name"] = json!(parts[2].as_str());
                        obj["names"] = json!([parts[1].as_str(), parts[2].as_str()]);
                    }
                }
                "titles" => {
                    obj["title"] = json!(parts[1].as_str());
                    if parts.len() >= 3 {
                        obj["plural_title"] = json!(parts[2].as_str());
                        obj["titles"] = json!([parts[1].as_str(), parts[2].as_str()]);
                    }
                }
                "description" => {
                    obj["description"] = json!(parts[1].as_str());
                }
                "d" => {
                    let pubkey_hex = event.pubkey.to_hex();
                    let d_val = parts[1].as_str();
                    obj["coordinate"] =
                        json!(format!("{}:{}:{}", event.kind.as_u16(), pubkey_hex, d_val));
                }
                _ => {}
            }
        }
    }

    obj
}

/// Sort JSON event objects by `created_at` descending, then `event_id` ascending.
pub fn sort_event_json_desc(events: &mut [serde_json::Value]) {
    events.sort_by(|a, b| {
        let a_created = a["created_at"].as_u64().unwrap_or(0);
        let b_created = b["created_at"].as_u64().unwrap_or(0);
        let a_id = a["event_id"].as_str().unwrap_or("");
        let b_id = b["event_id"].as_str().unwrap_or("");

        b_created.cmp(&a_created).then_with(|| a_id.cmp(b_id))
    });
}

/// Sort raw Event objects by `created_at` descending, then `id` ascending.
pub fn sort_events_desc(events: &mut [Event]) {
    events.sort_by(|a, b| {
        b.created_at
            .as_secs()
            .cmp(&a.created_at.as_secs())
            .then_with(|| a.id.to_hex().cmp(&b.id.to_hex()))
    });
}

/// Return a page of items from a slice.
pub fn paginate<T: Clone>(values: &[T], offset: usize, limit: usize) -> Vec<T> {
    if offset >= values.len() || limit == 0 {
        return Vec::new();
    }

    let end = offset.saturating_add(limit).min(values.len());
    values[offset..end].to_vec()
}

/// Extract the d-tag value from a header event's tags.
pub fn header_d_tag(event: &Event) -> Option<String> {
    event.tags.iter().find_map(|t| {
        let parts = t.as_slice();
        if parts.first().map(String::as_str) == Some("d") {
            parts.get(1).cloned()
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(kind: Kind, content: &str, tags: Vec<Tag>) -> Event {
        let keys = Keys::generate();
        EventBuilder::new(kind, content)
            .tags(tags)
            .sign_with_keys(&keys)
            .unwrap()
    }

    #[test]
    fn event_to_json_basic_fields() {
        let event = make_event(Kind::Custom(9998), "hello", vec![]);
        let j = event_to_json(&event);
        assert!(j["event_id"].is_string());
        assert_eq!(j["kind"], 9998);
        assert!(j["pubkey"].is_string());
        assert!(j["created_at"].is_number());
        assert!(j["sig"].is_string());
        assert_eq!(j["content"], "hello");
        assert!(j["tags"].is_array());
    }

    #[test]
    fn event_to_json_names_tag_extracts_singular_and_plural() {
        let tags = vec![Tag::custom(TagKind::custom("names"), ["mylist", "mylists"])];
        let event = make_event(Kind::Custom(9998), "", tags);
        let j = event_to_json(&event);
        assert_eq!(j["name"], "mylist");
        assert_eq!(j["plural_name"], "mylists");
        assert_eq!(j["names"], json!(["mylist", "mylists"]));
    }

    #[test]
    fn event_to_json_titles_tag_extracted() {
        let tags = vec![Tag::custom(
            TagKind::custom("titles"),
            ["My List", "My Lists"],
        )];
        let event = make_event(Kind::Custom(9998), "", tags);
        let j = event_to_json(&event);
        assert_eq!(j["title"], "My List");
        assert_eq!(j["plural_title"], "My Lists");
        assert_eq!(j["titles"], json!(["My List", "My Lists"]));
    }

    #[test]
    fn event_to_json_description_extracted() {
        let tags = vec![Tag::custom(
            TagKind::custom("description"),
            ["A description"],
        )];
        let event = make_event(Kind::Custom(9998), "", tags);
        let j = event_to_json(&event);
        assert_eq!(j["description"], "A description");
    }

    #[test]
    fn event_to_json_d_tag_creates_coordinate() {
        let keys = Keys::generate();
        let tags = vec![Tag::identifier("my-list")];
        let event = EventBuilder::new(Kind::Custom(39998), "")
            .tags(tags)
            .sign_with_keys(&keys)
            .unwrap();
        let j = event_to_json(&event);
        let coord = j["coordinate"].as_str().unwrap();
        assert!(coord.starts_with("39998:"));
        assert!(coord.ends_with(":my-list"));
        assert!(coord.contains(&keys.public_key().to_hex()));
    }

    #[test]
    fn event_to_json_unknown_tags_dont_pollute_top_level() {
        let tags = vec![Tag::custom(TagKind::custom("weird"), ["val"])];
        let event = make_event(Kind::Custom(9998), "", tags);
        let j = event_to_json(&event);
        assert!(j.get("weird").is_none());
    }

    #[test]
    fn paginate_returns_expected_window() {
        let values = vec![1, 2, 3, 4, 5];
        assert_eq!(paginate(&values, 1, 2), vec![2, 3]);
    }

    #[test]
    fn paginate_returns_empty_when_offset_out_of_range() {
        let values = vec![1, 2, 3];
        assert!(paginate(&values, 3, 10).is_empty());
    }

    #[test]
    fn paginate_returns_empty_when_limit_zero() {
        let values = vec![1, 2, 3];
        assert!(paginate(&values, 0, 0).is_empty());
    }

    #[test]
    fn sort_event_json_orders_by_created_at_desc_then_id() {
        let mut rows = vec![
            json!({"event_id": "b", "created_at": 100}),
            json!({"event_id": "a", "created_at": 100}),
            json!({"event_id": "c", "created_at": 120}),
        ];

        sort_event_json_desc(&mut rows);

        assert_eq!(rows[0]["event_id"], "c");
        assert_eq!(rows[1]["event_id"], "a");
        assert_eq!(rows[2]["event_id"], "b");
    }
}
