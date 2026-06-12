use stellartrail_domain::{gear::GearCategory, locale::Locale};
use stellartrail_importer::{Translator, parse_import_page};

#[test]
fn parses_8264_mobile_detail_page_facts() {
    let html = r#"
        <html>
          <head><title>探路者 38L 户外背包 - 8264手机触屏版</title></head>
          <body>
            <div class="starvalue">8.6</div>
            <div class="dpnum">7条点评</div>
            <section>分类：背包</section>
            <section>价格：￥349.00</section>
            <section>容量：38L
            推荐负重：12kg</section>
          </body>
        </html>
    "#;

    let parsed = parse_import_page(
        "https://m.8264.com/zhuangbei-equipmentDetail-2074165-1.html",
        html,
    )
    .expect("parse 8264");

    assert_eq!(parsed.source_key, "8264:2074165");
    assert_eq!(parsed.source_locale, Locale::ZhCn);
    assert_eq!(parsed.category, GearCategory::BackpackSystem);
    assert_eq!(parsed.name, "探路者 38L 户外背包");
    assert_eq!(parsed.official_price_cents, Some(34_900));
    assert_eq!(parsed.official_price_currency.as_deref(), Some("CNY"));
    assert_eq!(parsed.source_rating_score, Some(8.6));
    assert_eq!(parsed.source_rating_count, Some(7));
    assert_eq!(
        parsed.specs.get("capacity").map(String::as_str),
        Some("38L")
    );

    let draft = parsed
        .into_draft(
            "importer-user",
            "batch-fixture",
            &Translator::new("test").unwrap(),
        )
        .expect("draft");
    assert_eq!(draft.localizations.len(), 2);
    assert!(
        draft
            .localizations
            .iter()
            .any(|item| item.locale == Locale::En)
    );
}

#[test]
fn parses_8264_capacity_from_title_when_label_is_missing() {
    let html = r#"
        <html>
          <head><title>Toread/探路者男女通用38升户外双肩背包TEBC80681 - 8264手机触屏版</title></head>
          <body>
            <section>分类：背包</section>
            <section>价格：￥433.00</section>
          </body>
        </html>
    "#;

    let parsed = parse_import_page(
        "https://m.8264.com/zhuangbei-equipmentDetail-2074164-1.html",
        html,
    )
    .expect("parse 8264 capacity from title");

    assert_eq!(parsed.category, GearCategory::BackpackSystem);
    assert_eq!(
        parsed.specs.get("capacity").map(String::as_str),
        Some("38L")
    );
}

#[test]
fn parses_packwizard_sitemap_detail_page_facts() {
    let html = r#"
        <html>
          <head><meta property="og:title" content="Big Agnes Copper Spur HV UL2 Tent | PackWizard"></head>
          <body>
            <main>
              Category: tent
              Weight: 3 lb
              Capacity: 2 people
              MSRP $549.95
            </main>
          </body>
        </html>
    "#;

    let parsed = parse_import_page("https://packwizard.com/gear/tent/copper-spur-hv-ul2", html)
        .expect("parse packwizard");

    assert!(parsed.source_key.starts_with("packwizard:"));
    assert_eq!(parsed.source_locale, Locale::En);
    assert_eq!(parsed.category, GearCategory::SleepSystem);
    assert_eq!(parsed.name, "Big Agnes Copper Spur HV UL2 Tent");
    assert_eq!(parsed.weight_g, Some(1_361));
    assert_eq!(parsed.official_price_cents, Some(54_995));
    assert_eq!(parsed.official_price_currency.as_deref(), Some("USD"));
    assert_eq!(
        parsed.specs.get("people_count").map(String::as_str),
        Some("2 people")
    );
}

#[test]
fn parses_packwizard_app_shell_without_false_specs() {
    let html = r#"
        <html>
          <head>
            <meta property="og:title" content="Big Agnes Crag Lake SL2">
            <meta name="keywords" content="backpacking, ultralight gear, weight comparison">
            <title>PackWizard</title>
          </head>
          <body>
            <div id="root"></div>
            <script>window.__APP__ = { route: "/gear/tent/big-agnes-crag-lake-sl2" }</script>
          </body>
        </html>
    "#;

    let parsed = parse_import_page(
        "https://packwizard.com/gear/tent/big-agnes-crag-lake-sl2",
        html,
    )
    .expect("parse packwizard app shell");

    assert_eq!(parsed.category, GearCategory::SleepSystem);
    assert_eq!(parsed.name, "Big Agnes Crag Lake SL2");
    assert_eq!(parsed.weight_g, None);
    assert!(parsed.specs.is_empty());
}

#[test]
fn parses_trailspace_product_page_without_reviews() {
    let html = r#"
        <html>
          <head><meta property="og:title" content="Nemo Tensor Insulated Sleeping Pad Reviews - Trailspace"></head>
          <body>Sleeping pad Weight: 15 oz
          R-value: 4.2</body>
        </html>
    "#;

    let parsed = parse_import_page(
        "https://www.trailspace.com/gear/nemo/tensor-insulated/",
        html,
    )
    .expect("parse trailspace");

    assert!(parsed.source_key.starts_with("trailspace:"));
    assert_eq!(parsed.source_locale, Locale::En);
    assert_eq!(parsed.category, GearCategory::SleepSystem);
    assert_eq!(parsed.name, "Nemo Tensor Insulated Sleeping Pad");
    assert_eq!(parsed.weight_g, Some(425));
}

#[test]
fn english_source_generates_chinese_display_name_for_common_gear_terms() {
    let html = r#"
        <html>
          <head><meta property="og:title" content="Sierra Designs Backcountry Bivy"></head>
          <body>Bivy Weight: 13 oz</body>
        </html>
    "#;

    let parsed = parse_import_page(
        "https://www.outdoorgearreview.com/p/sierra-designs-backcountry-bivy/",
        html,
    )
    .expect("parse outdoor gear review");

    let draft = parsed
        .into_draft(
            "importer-user",
            "batch-fixture",
            &Translator::new("rule-based-test").unwrap(),
        )
        .expect("draft");
    let zh = draft
        .localizations
        .iter()
        .find(|localization| localization.locale == Locale::ZhCn)
        .expect("zh localization");
    assert_eq!(zh.name, "Sierra Designs Backcountry 露营袋");
    assert_eq!(zh.translation_status.as_deref(), Some("needs_review"));
    assert_eq!(
        zh.description.as_deref(),
        Some("待审核的外部导入装备条目，规格来自公开来源事实字段。")
    );
}

#[test]
fn parses_gearatlas_page_as_english_supplement() {
    let html = r#"
        <html>
          <head><title>Best Ultralight Backpacks | GearAtlas.com</title></head>
          <body>Backpack Weight: 900 g
          Capacity: 45 L</body>
        </html>
    "#;

    let parsed = parse_import_page("https://gearatlas.com/best-ultralight-backpacks/", html)
        .expect("parse gearatlas");

    assert!(parsed.source_key.starts_with("gearatlas:"));
    assert_eq!(parsed.source_locale, Locale::En);
    assert_eq!(parsed.category, GearCategory::BackpackSystem);
    assert_eq!(parsed.name, "Best Ultralight Backpacks");
    assert_eq!(parsed.weight_g, Some(900));
}

#[test]
fn parses_gearkr_candidate_page_as_low_confidence_chinese_source() {
    let html = r#"
        <html>
          <head><title>轻量头灯参数表 | GearKr</title></head>
          <body>头灯 亮度：400流明 重量：56g</body>
        </html>
    "#;

    let parsed = parse_import_page("http://gearkr.com/?p=123", html).expect("parse gearkr");

    assert!(parsed.source_key.starts_with("gearkr:"));
    assert_eq!(parsed.source_locale, Locale::ZhCn);
    assert_eq!(parsed.category, GearCategory::LightingSystem);
    assert_eq!(parsed.name, "轻量头灯参数表");
}

#[test]
fn parses_outdoor_gear_review_probe_page_conservatively() {
    let html = r#"
        <html>
          <head><title>Lightweight Backpack Review</title></head>
          <body>Backpack Weight: 2 lb
          Capacity: 40 L</body>
        </html>
    "#;

    let parsed = parse_import_page("https://www.outdoorgearreview.com/backpack-review/", html)
        .expect("parse outdoor gear review");

    assert!(parsed.source_key.starts_with("outdoorgearreview:"));
    assert_eq!(parsed.source_locale, Locale::En);
    assert_eq!(parsed.category, GearCategory::BackpackSystem);
    assert_eq!(parsed.weight_g, Some(907));
}
