use nostr::prelude::*;

use crate::error::DcoslError;

/// Parse a coordinate string of the form `kind:pubkey:d-tag`.
///
/// Returns `(kind_num, PublicKey, d_tag)`.
pub fn parse_coordinate_str(input: &str) -> Result<(u16, PublicKey, String), DcoslError> {
    let parts: Vec<&str> = input.splitn(3, ':').collect();
    if parts.len() != 3 {
        return Err(DcoslError::InvalidCoordinate {
            input: input.to_string(),
        });
    }
    let kind_num: u16 = parts[0]
        .parse()
        .map_err(|_| DcoslError::InvalidCoordinate {
            input: input.to_string(),
        })?;
    let pubkey = PublicKey::parse(parts[1]).map_err(|_| DcoslError::InvalidCoordinate {
        input: input.to_string(),
    })?;
    let d_tag = parts[2].to_string();
    Ok((kind_num, pubkey, d_tag))
}

pub struct ItemParams {
    pub header: Option<String>,
    pub header_coordinate: Option<String>,
    pub resource: String,
    pub content: Option<String>,
    pub fields: Vec<String>,
    pub addressable: bool,
    pub d_tag: Option<String>,
}

pub fn build_item_tags(
    parent_z_ref: &str,
    resource: &str,
    fields: &[String],
    d_tag: Option<&str>,
    client_name: Option<&str>,
) -> Vec<Tag> {
    let mut event_tags: Vec<Tag> = Vec::new();
    event_tags.push(Tag::custom(TagKind::custom("z"), [parent_z_ref]));
    event_tags.push(Tag::custom(TagKind::custom("r"), [resource]));

    if let Some(client) = client_name {
        event_tags.push(Tag::custom(TagKind::custom("client"), [client]));
    }

    event_tags.extend(fields.iter().filter_map(|field| {
        field
            .split_once('=')
            .map(|(key, val)| Tag::custom(TagKind::custom(key), [val]))
    }));

    if let Some(d) = d_tag {
        event_tags.push(Tag::identifier(d));
    }

    event_tags
}

/// Validate that either `header` or `header_coordinate` is provided.
pub fn validate_item_params(params: &ItemParams) -> Result<(), String> {
    if params.header.is_none() && params.header_coordinate.is_none() {
        return Err(
            "Specify --header=<event-id> or --header-coordinate=<kind:pubkey:d-tag>".to_string(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // parse_coordinate_str
    // -----------------------------------------------------------------------

    fn test_pubkey_hex() -> String {
        Keys::generate().public_key().to_hex()
    }

    #[test]
    fn parse_coordinate_valid() {
        let pk = test_pubkey_hex();
        let input = format!("39998:{pk}:my-list");
        let (kind, pubkey, d_tag) = parse_coordinate_str(&input).unwrap();
        assert_eq!(kind, 39998);
        assert_eq!(pubkey.to_hex(), pk);
        assert_eq!(d_tag, "my-list");
    }

    #[test]
    fn parse_coordinate_too_few_parts() {
        let err = parse_coordinate_str("39998:abc").unwrap_err();
        assert_eq!(err.code(), "INVALID_COORDINATE");
    }

    #[test]
    fn parse_coordinate_single_part() {
        let err = parse_coordinate_str("just-one").unwrap_err();
        assert_eq!(err.code(), "INVALID_COORDINATE");
    }

    #[test]
    fn parse_coordinate_invalid_kind() {
        let pk = test_pubkey_hex();
        let input = format!("notnum:{pk}:d");
        let err = parse_coordinate_str(&input).unwrap_err();
        assert_eq!(err.code(), "INVALID_COORDINATE");
    }

    #[test]
    fn parse_coordinate_invalid_pubkey() {
        let err = parse_coordinate_str("39998:not-a-pubkey:d").unwrap_err();
        assert_eq!(err.code(), "INVALID_COORDINATE");
    }

    #[test]
    fn parse_coordinate_d_tag_with_colons_preserved() {
        let pk = test_pubkey_hex();
        let input = format!("39998:{pk}:d:tag:with:colons");
        let (_, _, d_tag) = parse_coordinate_str(&input).unwrap();
        assert_eq!(d_tag, "d:tag:with:colons");
    }

    #[test]
    fn parse_coordinate_empty_d_tag() {
        let pk = test_pubkey_hex();
        let input = format!("39998:{pk}:");
        let (_, _, d_tag) = parse_coordinate_str(&input).unwrap();
        assert_eq!(d_tag, "");
    }

    // -----------------------------------------------------------------------
    // validate_item_params
    // -----------------------------------------------------------------------

    fn base_params(header: Option<String>, header_coordinate: Option<String>) -> ItemParams {
        ItemParams {
            header,
            header_coordinate,
            resource: "https://example.com".into(),
            content: None,
            fields: vec![],
            addressable: false,
            d_tag: None,
        }
    }

    #[test]
    fn validate_header_only_ok() {
        let p = base_params(Some("abc123".into()), None);
        assert!(validate_item_params(&p).is_ok());
    }

    #[test]
    fn validate_coordinate_only_ok() {
        let p = base_params(None, Some("39998:pk:d".into()));
        assert!(validate_item_params(&p).is_ok());
    }

    #[test]
    fn validate_neither_header_nor_coordinate_errors() {
        let p = base_params(None, None);
        assert!(validate_item_params(&p).is_err());
    }

    // -----------------------------------------------------------------------
    // build_item_tags
    // -----------------------------------------------------------------------

    fn find_tag<'a>(tags: &'a [Tag], kind_str: &str) -> Option<&'a Tag> {
        tags.iter()
            .find(|t| t.as_slice().first().map(String::as_str) == Some(kind_str))
    }

    fn tag_values(tag: &Tag) -> Vec<String> {
        tag.as_slice().iter().map(ToString::to_string).collect()
    }

    #[test]
    fn build_item_tags_has_parent_z_ref() {
        let tags = build_item_tags("abc123", "https://example.com", &[], None, None);
        let z = find_tag(&tags, "z").expect("z tag missing");
        assert_eq!(tag_values(z), vec!["z", "abc123"]);
    }

    #[test]
    fn build_item_tags_has_resource() {
        let tags = build_item_tags("abc123", "https://example.com", &[], None, None);
        let r = find_tag(&tags, "r").expect("r tag missing");
        assert_eq!(tag_values(r), vec!["r", "https://example.com"]);
    }

    #[test]
    fn build_item_tags_has_client_when_set() {
        let tags = build_item_tags("abc123", "https://example.com", &[], None, Some("test"));
        let c = find_tag(&tags, "client").expect("client tag missing");
        assert_eq!(tag_values(c), vec!["client", "test"]);
    }

    #[test]
    fn build_item_tags_no_client_when_none() {
        let tags = build_item_tags("abc123", "https://example.com", &[], None, None);
        assert!(find_tag(&tags, "client").is_none());
    }

    #[test]
    fn build_item_tags_fields_with_equals_become_tags() {
        let fields = vec!["color=red".to_string(), "size=large".to_string()];
        let tags = build_item_tags("abc123", "https://example.com", &fields, None, None);
        let color = find_tag(&tags, "color").expect("color tag missing");
        assert_eq!(tag_values(color), vec!["color", "red"]);
        let size = find_tag(&tags, "size").expect("size tag missing");
        assert_eq!(tag_values(size), vec!["size", "large"]);
    }

    #[test]
    fn build_item_tags_fields_without_equals_skipped() {
        let fields = vec!["no-equals-here".to_string()];
        let tags = build_item_tags("abc123", "https://example.com", &fields, None, None);
        // Should only have z, r — no extra tag (no client)
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn build_item_tags_d_tag_present() {
        let tags = build_item_tags("abc123", "https://example.com", &[], Some("my-item"), None);
        let d = find_tag(&tags, "d").expect("d tag missing");
        assert_eq!(tag_values(d), vec!["d", "my-item"]);
    }

    #[test]
    fn build_item_tags_d_tag_absent() {
        let tags = build_item_tags("abc123", "https://example.com", &[], None, None);
        assert!(find_tag(&tags, "d").is_none());
    }
}
