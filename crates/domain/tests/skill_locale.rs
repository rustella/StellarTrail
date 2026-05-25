use stellartrail_domain::skill::Locale;

#[test]
fn locale_parse_accepts_supported_header_aliases_only() {
    assert_eq!(Locale::parse("zh-CN"), Some(Locale::ZhCn));
    assert_eq!(Locale::parse("zh_Hans_CN"), Some(Locale::ZhCn));
    assert_eq!(Locale::parse("en-US"), Some(Locale::En));
    assert_eq!(Locale::parse("de-DE"), None);
    assert_eq!(Locale::parse("zh%2DCN"), None);
}

#[test]
fn locale_serializes_to_public_bcp47_tags() {
    assert_eq!(serde_json::to_string(&Locale::ZhCn).unwrap(), "\"zh-CN\"");
    assert_eq!(serde_json::to_string(&Locale::En).unwrap(), "\"en\"");
}
