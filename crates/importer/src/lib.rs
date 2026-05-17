//! Knots3D metadata importer helpers that convert exported JSON into DB import seeds.

use std::{fs, path::Path};

use anyhow::Context;
use serde_json::Value;
use stellartrail_domain::skill::{
    KnotCategorySeed, KnotLocalizationSeed, KnotMediaAssetSeed, KnotSeed, KnotTypeSeed, Locale,
};

/// Reads Knots3D bilingual metadata JSON from disk and converts it into DB import seeds.
pub fn read_knots3d_metadata(path: impl AsRef<Path>) -> anyhow::Result<Vec<KnotSeed>> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read Knots3D metadata {}", path.display()))?;
    parse_knots3d_metadata(&content)
        .with_context(|| format!("failed to parse Knots3D metadata {}", path.display()))
}

/// Parses Knots3D bilingual metadata JSON into internal import seeds.
pub fn parse_knots3d_metadata(raw: &str) -> anyhow::Result<Vec<KnotSeed>> {
    let value: Value = serde_json::from_str(raw).context("metadata must be JSON")?;
    let items = value
        .get("items")
        .and_then(Value::as_array)
        .context("metadata JSON must contain items array")?;

    let mut seeds = Vec::with_capacity(items.len());
    for item in items {
        let id = text(item, "id")
            .or_else(|| text(item, "english_slug"))
            .context("metadata item missing id")?;
        let english_slug = text(item, "english_slug").unwrap_or_else(|| id.clone());
        let zh_slug = text(item, "zh_slug");
        let en_language = item.pointer("/languages/en");
        let zh_language = item.pointer("/languages/zh-CN");

        let english_title = text(item, "english_name")
            .or_else(|| nested_text(en_language, "h1"))
            .or_else(|| nested_text(en_language, "title"))
            .unwrap_or_else(|| english_slug.clone());
        let chinese_title = text(item, "chinese_name")
            .or_else(|| nested_text(zh_language, "h1"))
            .or_else(|| zh_slug.clone())
            .unwrap_or_else(|| english_title.clone());
        let english_summary = text(item, "english_summary")
            .or_else(|| nested_text(en_language, "summary"))
            .or_else(|| {
                nested_text(en_language, "meta_description").and_then(|s| strip_title_prefix(&s))
            })
            .unwrap_or_default();
        let chinese_summary = text(item, "chinese_summary")
            .or_else(|| nested_text(zh_language, "summary"))
            .or_else(|| {
                nested_text(zh_language, "meta_description").and_then(|s| strip_title_prefix(&s))
            })
            .unwrap_or_else(|| english_summary.clone());
        let english_url = text(item, "english_url").or_else(|| nested_text(en_language, "url"));
        let chinese_slug = zh_slug.clone().unwrap_or_else(|| chinese_title.clone());

        seeds.push(KnotSeed {
            id: id.clone(),
            source_name: "Knots 3D".to_owned(),
            source_url: english_url,
            source_slug_en: english_slug.clone(),
            source_slug_zh: zh_slug,
            difficulty: None,
            localizations: vec![
                KnotLocalizationSeed {
                    locale: Locale::En,
                    slug: english_slug,
                    title: english_title,
                    summary: english_summary,
                    description: section_description(en_language),
                    steps: section_steps(en_language),
                },
                KnotLocalizationSeed {
                    locale: Locale::ZhCn,
                    slug: chinese_slug,
                    title: chinese_title,
                    summary: chinese_summary,
                    description: section_description(zh_language),
                    steps: section_steps(zh_language),
                },
            ],
            categories: parse_categories(item),
            types: parse_types(item),
            media: parse_media(item, &id),
            raw_metadata: item.clone(),
        });
    }

    Ok(seeds)
}

fn text(item: &Value, key: &str) -> Option<String> {
    item.get(key)?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn nested_text(item: Option<&Value>, key: &str) -> Option<String> {
    item?
        .get(key)?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn strip_title_prefix(value: &str) -> Option<String> {
    value
        .split_once(':')
        .map(|(_, rest)| rest.trim().to_owned())
        .filter(|s| !s.is_empty())
}

fn section_description(language: Option<&Value>) -> Option<String> {
    let sections = language?.get("sections")?.as_array()?;
    sections
        .iter()
        .filter_map(|section| section.get("text")?.as_str())
        .map(str::trim)
        .find(|text| !text.is_empty())
        .map(ToOwned::to_owned)
}

fn section_steps(language: Option<&Value>) -> Vec<String> {
    // Knots3D page sections are narrative metadata such as Usage, Warning, Related,
    // and ABOK. Their headings are not tying instructions, so only trust an
    // explicit step-like array when the metadata exporter provides one.
    language
        .and_then(|value| value.get("steps"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(step_text)
        .collect()
}

fn step_text(step: &Value) -> Option<String> {
    match step {
        Value::String(value) => trimmed_text(value),
        Value::Object(_) => step
            .get("text")
            .or_else(|| step.get("description"))
            .or_else(|| step.get("instruction"))
            .and_then(Value::as_str)
            .and_then(trimmed_text),
        _ => None,
    }
}

fn trimmed_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn parse_categories(item: &Value) -> Vec<KnotCategorySeed> {
    item.get("categories")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|category| {
            taxonomy_seed(category).map(|(id, en, zh)| KnotCategorySeed {
                id: id.clone(),
                localizations: vec![(Locale::En, id.clone(), en), (Locale::ZhCn, id, zh)],
            })
        })
        .collect()
}

fn parse_types(item: &Value) -> Vec<KnotTypeSeed> {
    item.get("types")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|knot_type| {
            taxonomy_seed(knot_type).map(|(id, en, zh)| KnotTypeSeed {
                id: id.clone(),
                localizations: vec![(Locale::En, id.clone(), en), (Locale::ZhCn, id, zh)],
            })
        })
        .collect()
}

fn taxonomy_seed(value: &Value) -> Option<(String, String, String)> {
    let id = text(value, "slug")?;
    let en = text(value, "en").unwrap_or_else(|| id.clone());
    let zh = text(value, "zh").unwrap_or_else(|| en.clone());
    Some((id, en, zh))
}

fn parse_media(item: &Value, knot_id: &str) -> Vec<KnotMediaAssetSeed> {
    let Some(media) = item.get("local_media") else {
        return Vec::new();
    };
    let mapping = [
        (
            "local_thumbnail",
            "thumbnail",
            "thumbnail.webp",
            "image/webp",
        ),
        ("local_preview", "preview", "preview.webp", "image/webp"),
        ("local_draw_gif", "draw_gif", "draw.gif", "image/gif"),
        (
            "local_360_gif",
            "turntable_gif",
            "turntable.gif",
            "image/gif",
        ),
        ("local_draw_mp4", "draw_mp4", "draw.mp4", "video/mp4"),
        (
            "local_360_mp4",
            "turntable_mp4",
            "turntable.mp4",
            "video/mp4",
        ),
    ];
    mapping
        .iter()
        .filter_map(|(source_key, media_type, filename, mime_type)| {
            let _source_path = media.get(*source_key)?.as_str()?;
            Some(KnotMediaAssetSeed {
                id: (*media_type).to_owned(),
                media_type: (*media_type).to_owned(),
                path: format!("skills/knots/{knot_id}/{filename}"),
                mime_type: (*mime_type).to_owned(),
                width: None,
                height: None,
                attribution: Some("Knots 3D".to_owned()),
                license_note: Some("Use only after authorization is confirmed.".to_owned()),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_knots3d_metadata_into_seed() {
        let seeds = parse_knots3d_metadata(
            r#"{
                "items": [{
                    "id": "bowline-knot",
                    "english_slug": "bowline-knot",
                    "zh_slug": "cheng-ren-jie",
                    "english_name": "Bowline",
                    "chinese_name": "称人结",
                    "categories": [{"slug": "camping", "en": "Camping", "zh": "露营"}],
                    "types": [{"slug": "loop", "en": "Loop", "zh": "绳圈"}],
                    "local_media": {"local_thumbnail": "/tmp/thumb.webp"},
                    "languages": {
                        "en": {"sections": [{"heading": "Step 1", "text": "Make a loop."}]},
                        "zh-CN": {"sections": [{"heading": "第 1 步", "text": "绕出一个绳圈。"}]}
                    }
                }]
            }"#,
        )
        .expect("parse metadata");

        assert_eq!(seeds.len(), 1);
        assert_eq!(seeds[0].id, "bowline-knot");
        assert_eq!(seeds[0].categories[0].id, "camping");
        assert_eq!(seeds[0].media[0].id, "thumbnail");
    }
}
