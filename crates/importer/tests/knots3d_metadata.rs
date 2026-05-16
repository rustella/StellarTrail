use stellartrail_domain::skill::Locale;
use stellartrail_importer::parse_knots3d_metadata;

#[test]
fn parses_compact_knots3d_metadata_into_internal_seed_without_public_slug_leakage() {
    let raw = r#"
    {
      "items": [
        {
          "id": "adjustable-grip-hitch-knot",
          "english_slug": "adjustable-grip-hitch-knot",
          "zh_slug": "可调节绳结",
          "english_name": "Adjustable Grip Hitch",
          "chinese_name": "可调节绳结",
          "english_summary": "Adjust tension on a line.",
          "chinese_summary": "调节绳索上的张力。",
          "english_url": "https://knots3d.com/en/adjustable-grip-hitch-knot",
          "chinese_url": "https://knots3d.com/zh-CN/%E5%8F%AF%E8%B0%83%E8%8A%82%E7%BB%B3%E7%BB%93",
          "categories": [{"slug":"camping-knots","en":"Camping","zh":"露营"}],
          "types": [{"slug":"hitch-knots","en":"Hitches","zh":"套结"}],
          "local_media": {
            "local_thumbnail": "thumbnails/adjustable-grip-hitch-knot.webp",
            "local_preview": "previews/adjustable-grip-hitch-knot.webp"
          }
        }
      ]
    }
    "#;

    let seeds = parse_knots3d_metadata(raw).expect("parse");
    assert_eq!(seeds.len(), 1);
    let knot = &seeds[0];
    assert_eq!(knot.id, "adjustable-grip-hitch-knot");
    assert_eq!(knot.source_slug_en, "adjustable-grip-hitch-knot");
    assert_eq!(knot.source_slug_zh.as_deref(), Some("可调节绳结"));
    assert_eq!(
        knot.localizations
            .iter()
            .find(|l| l.locale == Locale::ZhCn)
            .unwrap()
            .title,
        "可调节绳结"
    );
    assert_eq!(
        knot.localizations
            .iter()
            .find(|l| l.locale == Locale::En)
            .unwrap()
            .summary,
        "Adjust tension on a line."
    );
    assert_eq!(knot.categories[0].id, "camping-knots");
    assert_eq!(knot.types[0].id, "hitch-knots");
    assert!(knot
        .media
        .iter()
        .any(|m| m.path == "skills/knots/adjustable-grip-hitch-knot/thumbnail.webp"));
}
