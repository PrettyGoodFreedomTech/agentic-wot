use nostr::prelude::*;

#[derive(Debug, Clone)]
pub struct HeaderParams {
    pub name: String,
    pub plural_name: String,
    pub titles: Vec<String>,
    pub description: Option<String>,
    pub required: Vec<String>,
    pub recommended: Vec<String>,
    pub tags_list: Vec<String>,
    pub alt: Option<String>,
    pub d_tag: Option<String>,
    /// Client identifier tag (e.g. "wokhei", "magic-carpet").
    pub client_name: Option<String>,
}

pub fn build_header_tags(params: &HeaderParams) -> Vec<Tag> {
    let HeaderParams {
        name,
        plural_name,
        titles,
        description,
        required,
        recommended,
        tags_list,
        alt,
        d_tag,
        client_name,
    } = params;

    let mut event_tags: Vec<Tag> = Vec::new();

    event_tags.push(Tag::custom(
        TagKind::custom("names"),
        [name.clone(), plural_name.clone()],
    ));

    if titles.len() == 2 {
        event_tags.push(Tag::custom(TagKind::custom("titles"), titles.clone()));
    }

    if let Some(desc) = description {
        event_tags.push(Tag::custom(TagKind::custom("description"), [desc.clone()]));
    }
    if !required.is_empty() {
        event_tags.push(Tag::custom(TagKind::custom("required"), required.clone()));
    }
    event_tags.extend(
        recommended
            .iter()
            .map(|field| Tag::custom(TagKind::custom("recommended"), [field.clone()])),
    );
    event_tags.extend(tags_list.iter().map(Tag::hashtag));

    let alt_text = alt
        .clone()
        .unwrap_or_else(|| format!("DCoSL list header: {name} / {plural_name}"));
    event_tags.push(Tag::custom(TagKind::custom("alt"), [alt_text]));

    if let Some(client) = client_name {
        event_tags.push(Tag::custom(TagKind::custom("client"), [client.clone()]));
    }

    if let Some(d) = d_tag {
        event_tags.push(Tag::identifier(d));
    }

    event_tags
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_params() -> HeaderParams {
        HeaderParams {
            name: "mylist".into(),
            plural_name: "mylists".into(),
            titles: vec![],
            description: None,
            required: vec![],
            recommended: vec![],
            tags_list: vec![],
            alt: None,
            d_tag: None,
            client_name: Some("test-client".into()),
        }
    }

    fn find_tag<'a>(tags: &'a [Tag], kind_str: &str) -> Option<&'a Tag> {
        tags.iter()
            .find(|t| t.as_slice().first().map(String::as_str) == Some(kind_str))
    }

    fn find_tags<'a>(tags: &'a [Tag], kind_str: &str) -> Vec<&'a Tag> {
        tags.iter()
            .filter(|t| t.as_slice().first().map(String::as_str) == Some(kind_str))
            .collect()
    }

    fn tag_values(tag: &Tag) -> Vec<String> {
        tag.as_slice().iter().map(ToString::to_string).collect()
    }

    #[test]
    fn minimal_params_has_names_tag_with_singular_and_plural() {
        let tags = build_header_tags(&minimal_params());
        let names = find_tag(&tags, "names").expect("names tag missing");
        assert_eq!(tag_values(names), vec!["names", "mylist", "mylists"]);
    }

    #[test]
    fn titles_tag_absent_when_not_set() {
        let tags = build_header_tags(&minimal_params());
        assert!(find_tag(&tags, "titles").is_none());
    }

    #[test]
    fn titles_tag_present_when_set() {
        let mut p = minimal_params();
        p.titles = vec!["My List".into(), "My Lists".into()];
        let tags = build_header_tags(&p);
        let titles = find_tag(&tags, "titles").expect("titles tag missing");
        assert_eq!(tag_values(titles), vec!["titles", "My List", "My Lists"]);
    }

    #[test]
    fn minimal_params_has_alt_tag_with_default() {
        let tags = build_header_tags(&minimal_params());
        let alt = find_tag(&tags, "alt").expect("alt tag missing");
        let vals = tag_values(alt);
        assert!(vals[1].contains("mylist"));
        assert!(vals[1].contains("mylists"));
    }

    #[test]
    fn minimal_params_has_client_tag() {
        let tags = build_header_tags(&minimal_params());
        let client = find_tag(&tags, "client").expect("client tag missing");
        assert_eq!(tag_values(client), vec!["client", "test-client"]);
    }

    #[test]
    fn description_present_when_set() {
        let mut p = minimal_params();
        p.description = Some("A description".into());
        let tags = build_header_tags(&p);
        let desc = find_tag(&tags, "description").expect("description tag missing");
        assert_eq!(tag_values(desc), vec!["description", "A description"]);
    }

    #[test]
    fn description_absent_when_none() {
        let tags = build_header_tags(&minimal_params());
        assert!(find_tag(&tags, "description").is_none());
    }

    #[test]
    fn required_fields_present() {
        let mut p = minimal_params();
        p.required = vec!["url".into(), "name".into()];
        let tags = build_header_tags(&p);
        let req = find_tag(&tags, "required").expect("required tag missing");
        assert_eq!(tag_values(req), vec!["required", "url", "name"]);
    }

    #[test]
    fn recommended_fields_emitted_as_separate_tags() {
        let mut p = minimal_params();
        p.recommended = vec!["desc".into(), "operator".into()];
        let tags = build_header_tags(&p);
        let rec_tags = find_tags(&tags, "recommended");
        assert_eq!(rec_tags.len(), 2);
        assert_eq!(tag_values(rec_tags[0]), vec!["recommended", "desc"]);
        assert_eq!(tag_values(rec_tags[1]), vec!["recommended", "operator"]);
    }

    #[test]
    fn hashtags_generated_from_tags_list() {
        let mut p = minimal_params();
        p.tags_list = vec!["nostr".into(), "dcosl".into()];
        let tags = build_header_tags(&p);
        let t_tags: Vec<_> = tags
            .iter()
            .filter(|t| t.as_slice().first().map(String::as_str) == Some("t"))
            .collect();
        assert_eq!(t_tags.len(), 2);
    }

    #[test]
    fn custom_alt_text_overrides_default() {
        let mut p = minimal_params();
        p.alt = Some("Custom alt".into());
        let tags = build_header_tags(&p);
        let alt = find_tag(&tags, "alt").unwrap();
        assert_eq!(tag_values(alt), vec!["alt", "Custom alt"]);
    }

    #[test]
    fn d_tag_adds_identifier() {
        let mut p = minimal_params();
        p.d_tag = Some("my-id".into());
        let tags = build_header_tags(&p);
        let d = find_tag(&tags, "d").expect("d tag missing");
        assert_eq!(tag_values(d), vec!["d", "my-id"]);
    }

    #[test]
    fn no_d_tag_when_none() {
        let tags = build_header_tags(&minimal_params());
        assert!(find_tag(&tags, "d").is_none());
    }

    #[test]
    fn no_client_tag_when_none() {
        let mut p = minimal_params();
        p.client_name = None;
        let tags = build_header_tags(&p);
        assert!(find_tag(&tags, "client").is_none());
    }
}
