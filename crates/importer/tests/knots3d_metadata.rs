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
          "zh_slug": "ke-tiao-jie-sheng-jie",
          "english_name": "Adjustable Grip Hitch",
          "chinese_name": "可调节绳结",
          "english_summary": "Adjust tension on a line.",
          "chinese_summary": "调节绳索上的张力。",
          "english_url": "https://knots3d.com/en/adjustable-grip-hitch-knot",
          "chinese_url": "https://knots3d.com/zh-CN/ke-tiao-jie-sheng-jie",
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
    assert_eq!(
        knot.source_slug_zh.as_deref(),
        Some("ke-tiao-jie-sheng-jie")
    );
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
    assert!(
        knot.localizations
            .iter()
            .find(|l| l.locale == Locale::En)
            .unwrap()
            .aliases
            .is_empty()
    );
    assert_eq!(knot.categories[0].id, "camping-knots");
    assert_eq!(knot.types[0].id, "hitch-knots");
    assert!(
        knot.media
            .iter()
            .any(|m| m.path == "skills/knots/adjustable-grip-hitch-knot/thumbnail.webp")
    );
}

#[test]
fn parses_knots3d_aliases_from_language_metadata() {
    let raw = r#"
    {
      "items": [
        {
          "id": "adjustable-grip-hitch-knot",
          "english_slug": "adjustable-grip-hitch-knot",
          "zh_slug": "ke-tiao-jie-sheng-jie",
          "english_name": "Adjustable Grip Hitch",
          "chinese_name": "可调节绳结",
          "meta_keywords": ["ignored keyword"],
          "subtitle": "ignored subtitle",
          "languages": {
            "en": {
              "aliases": [
                " Adjustable Loop ",
                "Cawley   Adjustable Hitch",
                "adjustable loop",
                "Adjustable Grip Hitch",
                "",
                42
              ]
            },
            "zh-CN": {
              "aliases": [
                " 可调节活结 ",
                "考利   可调节套结",
                "可调节活结",
                "可调节绳结"
              ]
            }
          }
        }
      ]
    }
    "#;

    let seeds = parse_knots3d_metadata(raw).expect("parse");
    let en = seeds[0]
        .localizations
        .iter()
        .find(|localization| localization.locale == Locale::En)
        .expect("en localization");
    let zh = seeds[0]
        .localizations
        .iter()
        .find(|localization| localization.locale == Locale::ZhCn)
        .expect("zh localization");

    assert_eq!(
        en.aliases,
        vec![
            "Adjustable Loop".to_owned(),
            "Cawley Adjustable Hitch".to_owned()
        ]
    );
    assert_eq!(
        zh.aliases,
        vec!["可调节活结".to_owned(), "考利 可调节套结".to_owned()]
    );
}

#[test]
fn ignores_knots3d_section_headings_when_no_explicit_steps_exist() {
    let raw = r#"
    {
      "items": [
        {
          "id": "ashleys-bend-knot",
          "english_slug": "ashleys-bend-knot",
          "zh_slug": "a-shi-li-jie",
          "english_name": "Ashley's Bend",
          "chinese_name": "阿什利结",
          "english_summary": "Join two lines of stiff, slippery material.",
          "chinese_summary": "连接两条坚硬、光滑材质的绳索。",
          "languages": {
            "zh-CN": {
              "sections": [
                {"heading": "用途", "text": "阿什利结用于连接两条绳子。"},
                {"heading": "警告 ⚠️", "text": "和经验丰富的指导人员验证绑扎技术。"},
                {"heading": "相关", "text": ""},
                {"heading": "ABOK", "text": ""}
              ]
            },
            "en": {
              "sections": [
                {"heading": "Usage", "text": "Join two ropes."},
                {"heading": "Warning ⚠️", "text": "Verify tying technique."}
              ]
            }
          }
        }
      ]
    }
    "#;

    let seeds = parse_knots3d_metadata(raw).expect("parse");
    let zh = seeds[0]
        .localizations
        .iter()
        .find(|localization| localization.locale == Locale::ZhCn)
        .expect("zh localization");

    assert_eq!(
        zh.description.as_deref(),
        Some("阿什利结用于连接两条绳子。")
    );
    assert!(zh.steps.is_empty());
}

#[test]
fn parses_explicit_knots3d_steps_when_present() {
    let raw = r#"
    {
      "items": [
        {
          "id": "practice-knot",
          "english_slug": "practice-knot",
          "english_name": "Practice Knot",
          "languages": {
            "zh-CN": {
              "steps": [
                " 绕出一个绳圈。 ",
                {"text": "将绳头穿过绳圈。"},
                {"instruction": "收紧并检查受力。"},
                ""
              ]
            },
            "en": {
              "steps": ["Make a loop."]
            }
          }
        }
      ]
    }
    "#;

    let seeds = parse_knots3d_metadata(raw).expect("parse");
    let zh = seeds[0]
        .localizations
        .iter()
        .find(|localization| localization.locale == Locale::ZhCn)
        .expect("zh localization");

    assert_eq!(
        zh.steps,
        vec![
            "绕出一个绳圈。".to_owned(),
            "将绳头穿过绳圈。".to_owned(),
            "收紧并检查受力。".to_owned(),
        ]
    );
}
