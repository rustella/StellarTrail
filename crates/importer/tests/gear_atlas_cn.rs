use stellartrail_domain::gear::GearCategory;
use stellartrail_importer::gear_atlas_cn::{
    GearAtlasCnImportArgs, map_8264_category, parse_8264_mobile_gear_page, parse_url_file_content,
};

#[test]
fn parses_8264_mobile_detail_fact_fields_without_review_body() {
    let html = r#"
    <html>
      <body>
        <div class="contenttop">
          <div class="namebox">户外装备探路者38L户外背包双肩包TEBB9060</div>
          <i class="starvalue">8.6</i>
          <em class="dpnum">7点评</em>
        </div>
        <div class="feleibox">
          <ul>
            <li><span>中型背包（30L--45L）</span></li>
            <li><span>¥349.00</span></li>
          </ul>
        </div>
        <input type="hidden" name="tid" value="2074165"/>
        <div class="comment-word">
          <div class="attrValues">这段用户点评正文不应该进入导入记录。</div>
        </div>
      </body>
    </html>
    "#;

    let record = parse_8264_mobile_gear_page(
        html,
        "https://m.8264.com/zhuangbei-equipmentDetail-2074165-1.html",
    )
    .expect("parse 8264 page");

    assert_eq!(record.source_key, "8264:2074165");
    assert_eq!(record.name, "户外装备探路者38L户外背包双肩包TEBB9060");
    assert_eq!(
        record.source_category_label.as_deref(),
        Some("中型背包（30L--45L）")
    );
    assert_eq!(record.category, GearCategory::BackpackSystem);
    assert_eq!(record.official_price_cents, Some(34_900));
    assert_eq!(record.source_rating_score, Some(8.6));
    assert_eq!(record.source_rating_count, Some(7));

    let draft = record
        .into_external_import_draft("import-user".to_owned(), Some("batch-20260521".to_owned()));
    assert_eq!(
        draft.description.as_deref(),
        Some("来自 8264 户外用品点评的公开事实字段，已保留来源链接供审核。")
    );
    assert!(!draft.description.unwrap().contains("用户点评正文"));
}

#[test]
fn maps_first_pass_8264_categories() {
    assert_eq!(
        map_8264_category("中型背包（30L--45L）"),
        GearCategory::BackpackSystem
    );
    assert_eq!(map_8264_category("双人帐篷"), GearCategory::SleepSystem);
    assert_eq!(map_8264_category("炉具套锅"), GearCategory::KitchenSystem);
    assert_eq!(map_8264_category("登山杖"), GearCategory::WalkingSystem);
    assert_eq!(
        map_8264_category("登山/徒步杖"),
        GearCategory::WalkingSystem
    );
    assert_eq!(
        map_8264_category("冲锋衣服装"),
        GearCategory::ClothingSystem
    );
    assert_eq!(map_8264_category("头灯"), GearCategory::LightingSystem);
    assert_eq!(
        map_8264_category("GPS电子数码"),
        GearCategory::ElectronicsSystem
    );
    assert_eq!(
        map_8264_category("攀登器材安全带"),
        GearCategory::TechnicalGear
    );
    assert_eq!(map_8264_category("未知分类"), GearCategory::OtherGear);
}

#[test]
fn prefers_full_8264_title_when_mobile_namebox_is_truncated() {
    let html = r#"
    <html>
      <head>
        <title>BLACKDIAMOND(黑钻) 120-140登山杖 Distance FL Z-poles 112123 - 8264手机触屏版</title>
      </head>
      <body>
        <div class="namebox">BLACKDIAMOND(黑钻) 120-140登</div>
        <i class="starvalue">9.0</i>
        <em class="dpnum">4点评</em>
        <div class="feleibox">
          <ul>
            <li><span>登山/徒步杖</span></li>
            <li><span>¥799.00</span></li>
          </ul>
        </div>
      </body>
    </html>
    "#;

    let record = parse_8264_mobile_gear_page(
        html,
        "https://m.8264.com/zhuangbei-equipmentDetail-2081437-1.html",
    )
    .expect("parse truncated mobile page");

    assert_eq!(
        record.name,
        "BLACKDIAMOND(黑钻) 120-140登山杖 Distance FL Z-poles 112123"
    );
    assert_eq!(record.category, GearCategory::WalkingSystem);
}

#[test]
fn parses_url_files_and_cli_safety_defaults() {
    let urls = parse_url_file_content(
        r#"
        # first batch
        https://m.8264.com/zhuangbei-equipmentDetail-2074165-1.html

        https://m.8264.com/zhuangbei-2074166-1.html
        "#,
    );
    assert_eq!(urls.len(), 2);

    let dry_run = GearAtlasCnImportArgs::parse_from([
        "--url",
        "https://m.8264.com/zhuangbei-equipmentDetail-2074165-1.html",
    ])
    .expect("dry-run args");
    assert!(!dry_run.write);
    assert_eq!(dry_run.limit, 20);
    assert_eq!(dry_run.delay_ms, 1_500);

    let missing_submitter = GearAtlasCnImportArgs::parse_from([
        "--write",
        "--database-url",
        "sqlite://example.db?mode=rwc",
        "--url",
        "https://m.8264.com/zhuangbei-equipmentDetail-2074165-1.html",
    ]);
    assert!(missing_submitter.is_err());
}
