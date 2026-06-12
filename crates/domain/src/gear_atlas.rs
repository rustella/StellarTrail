//! Public gear atlas domain models and validation.
//!
//! Gear atlas records are public-market gear snapshots. They deliberately keep
//! a much smaller field set than personal gear records so user-specific
//! purchase, storage, status, and note data cannot leak into public reads.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    gear::{
        GearCategory, GearSpecs, GearVariant, GearVariants, SUPPORTED_CURRENCIES, normalize_specs,
        normalize_variants, now_rfc3339, variant_key_from_label,
    },
    locale::Locale,
    validation::{
        FieldViolation, ValidationError, normalize_optional_text, normalize_required_text,
    },
};

/// Review status for one public gear atlas submission.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GearAtlasStatus {
    #[default]
    Pending,
    Approved,
    Rejected,
}

impl GearAtlasStatus {
    pub const ALL: [Self; 3] = [Self::Pending, Self::Approved, Self::Rejected];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }

    pub fn from_key(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == value)
    }
}

impl std::fmt::Display for GearAtlasStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Source that created a gear atlas submission.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GearAtlasSourceType {
    #[default]
    Manual,
    UserGear,
    ExternalImport,
}

impl GearAtlasSourceType {
    pub const ALL: [Self; 3] = [Self::Manual, Self::UserGear, Self::ExternalImport];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::UserGear => "user_gear",
            Self::ExternalImport => "external_import",
        }
    }

    pub fn from_key(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|source_type| source_type.as_str() == value)
    }
}

impl std::fmt::Display for GearAtlasSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Sort options supported by public gear atlas list reads.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GearAtlasSort {
    #[default]
    ApprovedAtDesc,
    NameAsc,
    WeightDesc,
    OfficialPriceDesc,
}

/// Complete persisted atlas item, including review metadata.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearAtlasItem {
    pub id: String,
    pub category: GearCategory,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    pub variants: GearVariants,
    pub specs: GearSpecs,
    pub submitted_snapshot: GearAtlasPublicSnapshot,
    pub review_changes: GearAtlasReviewChanges,
    pub source_type: GearAtlasSourceType,
    pub submitted_by_user_id: String,
    pub source_user_gear_id: Option<String>,
    pub status: GearAtlasStatus,
    pub rejection_reason: Option<String>,
    pub reviewed_by_user_id: Option<String>,
    pub reviewed_at: Option<String>,
    pub approved_at: Option<String>,
    pub source_key: Option<String>,
    pub source_name: Option<String>,
    pub source_url: Option<String>,
    pub source_license_note: Option<String>,
    pub import_batch_id: Option<String>,
    pub imported_at: Option<String>,
    pub source_rating_score: Option<f64>,
    pub source_rating_count: Option<i32>,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Public fields as originally submitted for later review-change summaries.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GearAtlasPublicSnapshot {
    pub category: GearCategory,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    pub variants: GearVariants,
    pub specs: GearSpecs,
}

impl GearAtlasPublicSnapshot {
    /// Captures the submitted public fields after validation normalization.
    pub fn from_draft(draft: &GearAtlasDraft) -> Self {
        Self {
            category: draft.category,
            name: draft.name.clone(),
            brand: draft.brand.clone(),
            model: draft.model.clone(),
            description: draft.description.clone(),
            weight_g: draft.weight_g,
            official_price_cents: draft.official_price_cents,
            official_price_currency: draft.official_price_currency.clone(),
            variants: draft.variants.clone(),
            specs: draft.specs.clone(),
        }
    }

    /// Captures the current persisted public fields.
    pub fn from_item(item: &GearAtlasItem) -> Self {
        Self {
            category: item.category,
            name: item.name.clone(),
            brand: item.brand.clone(),
            model: item.model.clone(),
            description: item.description.clone(),
            weight_g: item.weight_g,
            official_price_cents: item.official_price_cents,
            official_price_currency: item.official_price_currency.clone(),
            variants: item.variants.clone(),
            specs: item.specs.clone(),
        }
    }
}

/// One user-visible field adjustment made during administrator review.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GearAtlasReviewChange {
    pub field: String,
    pub label: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

pub type GearAtlasReviewChanges = Vec<GearAtlasReviewChange>;

/// Writable public atlas draft created from a manual form or a personal gear snapshot.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearAtlasDraft {
    pub category: GearCategory,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    pub variants: GearVariants,
    pub specs: GearSpecs,
    pub source_type: GearAtlasSourceType,
    pub submitted_by_user_id: String,
    pub source_user_gear_id: Option<String>,
}

/// Writable draft created by a conservative external source import.
///
/// The public fields mirror a normal atlas submission, while source metadata is
/// kept separate so clients can audit where imported facts came from without
/// exposing crawler-only identifiers such as `source_key`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearAtlasExternalImportDraft {
    pub category: GearCategory,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub official_price_cents: Option<i64>,
    pub official_price_currency: Option<String>,
    pub variants: GearVariants,
    pub specs: GearSpecs,
    pub submitted_by_user_id: String,
    pub source_key: String,
    pub source_name: String,
    pub source_url: Option<String>,
    pub source_license_note: Option<String>,
    pub import_batch_id: Option<String>,
    pub source_rating_score: Option<f64>,
    pub source_rating_count: Option<i32>,
    pub canonical_key: Option<String>,
    pub source_locale: Option<Locale>,
    pub detail_score: Option<i32>,
    pub localizations: Vec<GearAtlasLocalizationDraft>,
}

/// Locale-specific public text and display facts produced by an external import.
///
/// The canonical atlas item keeps normalized numeric and source fields on the
/// main row. Localizations hold user-visible text for the requested API locale,
/// including localized variant labels and short spec values.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearAtlasLocalizationDraft {
    pub locale: Locale,
    pub name: String,
    pub description: Option<String>,
    pub variants: GearVariants,
    pub specs: GearSpecs,
    pub translation_status: Option<String>,
    pub translation_provider: Option<String>,
    pub translated_at: Option<String>,
}

/// Stable status stored when an admin has reviewed one localized atlas row.
pub const GEAR_ATLAS_LOCALIZATION_STATUS_REVIEWED: &str = "reviewed";

/// Stable status stored when a localized atlas row still needs human review.
pub const GEAR_ATLAS_LOCALIZATION_STATUS_NEEDS_REVIEW: &str = "needs_review";

/// Stable status stored when an admin saves localized content without approving it.
pub const GEAR_ATLAS_LOCALIZATION_STATUS_DRAFT: &str = "draft";

/// Review state computed for one locale in the administrator workflow.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GearAtlasLocalizationReviewState {
    Missing,
    Draft,
    NeedsReview,
    Reviewed,
}

/// Review summary for one locale in the administrator atlas workflow.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GearAtlasLocalizationReviewStatus {
    pub locale: Locale,
    pub state: GearAtlasLocalizationReviewState,
    pub missing_fields: Vec<String>,
    pub translation_status: Option<String>,
}

/// Deterministic localization helper for imported atlas display content.
///
/// This is deliberately small and auditable. It translates common gear category
/// words while preserving brands, models, numbers, units, and source metadata.
#[derive(Clone, Debug)]
pub struct GearAtlasLocalizationTranslator {
    provider: String,
}

impl GearAtlasLocalizationTranslator {
    pub fn new(provider: impl Into<String>) -> Option<Self> {
        let provider = provider.into();
        let provider = provider.trim();
        if provider.is_empty() {
            None
        } else {
            Some(Self {
                provider: provider.to_owned(),
            })
        }
    }

    /// Builds a generated display localization without copying source prose.
    pub fn translate_localization(
        &self,
        name: &str,
        variants: &GearVariants,
        specs: &GearSpecs,
        locale: Locale,
    ) -> GearAtlasLocalizationDraft {
        let translated = self.translate_fields(name, variants, specs, locale);
        GearAtlasLocalizationDraft {
            locale,
            name: translated.name,
            description: translated.description,
            variants: translated.variants,
            specs: translated.specs,
            translation_status: Some(translated.translation_status),
            translation_provider: Some(self.provider.clone()),
            translated_at: Some(now_rfc3339()),
        }
    }

    fn translate_fields(
        &self,
        name: &str,
        variants: &GearVariants,
        specs: &GearSpecs,
        locale: Locale,
    ) -> TranslatedAtlasLocalization {
        let name = match locale {
            Locale::ZhCn => translate_en_to_zh(name),
            Locale::En => translate_zh_to_en(name),
        };
        let description = Some(match locale {
            Locale::ZhCn => "待审核的外部导入装备条目，规格来自公开来源事实字段。".to_owned(),
            Locale::En => {
                "Pending external-import gear atlas item from public fact fields.".to_owned()
            }
        });
        let variants = variants
            .iter()
            .enumerate()
            .map(|(index, variant)| GearVariant {
                key: if variant.key.trim().is_empty() {
                    variant_key_from_label(&variant.label, index)
                } else {
                    variant.key.clone()
                },
                label: match locale {
                    Locale::ZhCn => translate_en_to_zh(&variant.label),
                    Locale::En => translate_zh_to_en(&variant.label),
                },
                official_price_cents: variant.official_price_cents,
                official_price_currency: variant.official_price_currency.clone(),
                weight_g: variant.weight_g,
            })
            .collect();
        let specs = specs
            .iter()
            .map(|(key, value)| {
                let translated = match locale {
                    Locale::ZhCn => translate_en_to_zh(value),
                    Locale::En => translate_zh_to_en(value),
                };
                (key.clone(), translated)
            })
            .collect();
        let translation_status = match locale {
            Locale::ZhCn => GEAR_ATLAS_LOCALIZATION_STATUS_NEEDS_REVIEW,
            Locale::En if name.chars().any(is_ascii_letter) => "translated",
            Locale::En => GEAR_ATLAS_LOCALIZATION_STATUS_NEEDS_REVIEW,
        }
        .to_owned();
        TranslatedAtlasLocalization {
            name,
            description,
            variants,
            specs,
            translation_status,
        }
    }
}

struct TranslatedAtlasLocalization {
    name: String,
    description: Option<String>,
    variants: GearVariants,
    specs: GearSpecs,
    translation_status: String,
}

impl GearAtlasDraft {
    /// Validates and normalizes only the public fields allowed in the atlas.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        self.name =
            normalize_required_text(std::mem::take(&mut self.name), 100, "name", &mut errors);
        self.brand = normalize_optional_text(self.brand.take(), 80, "brand", &mut errors);
        self.model = normalize_optional_text(self.model.take(), 80, "model", &mut errors);
        self.description =
            normalize_optional_text(self.description.take(), 100, "description", &mut errors);
        self.submitted_by_user_id = normalize_required_text(
            std::mem::take(&mut self.submitted_by_user_id),
            128,
            "submitted_by_user_id",
            &mut errors,
        );
        self.source_user_gear_id = normalize_optional_text(
            self.source_user_gear_id.take(),
            128,
            "source_user_gear_id",
            &mut errors,
        );

        if self
            .weight_g
            .is_some_and(|weight_g| !(0..=1_000_000).contains(&weight_g))
        {
            errors.push(FieldViolation::new(
                "weight_g",
                "must be between 0 and 1000000",
            ));
        }

        if self.official_price_cents.is_some_and(|price| price < 0) {
            errors.push(FieldViolation::new(
                "official_price_cents",
                "must be greater than or equal to 0",
            ));
        }
        self.official_price_currency = normalize_official_price_currency(
            self.official_price_cents,
            self.official_price_currency.take(),
            &mut errors,
        );
        self.variants = normalize_variants(std::mem::take(&mut self.variants), &mut errors);
        self.specs = normalize_specs(self.category, std::mem::take(&mut self.specs), &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

impl GearAtlasExternalImportDraft {
    /// Validates source-audit metadata and the public atlas fields that will be reviewed.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut public_draft = GearAtlasDraft {
            category: self.category,
            name: std::mem::take(&mut self.name),
            brand: self.brand.take(),
            model: self.model.take(),
            description: self.description.take(),
            weight_g: self.weight_g,
            official_price_cents: self.official_price_cents,
            official_price_currency: self.official_price_currency.take(),
            variants: std::mem::take(&mut self.variants),
            specs: std::mem::take(&mut self.specs),
            source_type: GearAtlasSourceType::ExternalImport,
            submitted_by_user_id: std::mem::take(&mut self.submitted_by_user_id),
            source_user_gear_id: None,
        };
        let mut errors = match public_draft.validate_and_normalize() {
            Ok(()) => Vec::new(),
            Err(error) => error.fields,
        };

        self.category = public_draft.category;
        self.name = public_draft.name;
        self.brand = public_draft.brand;
        self.model = public_draft.model;
        self.description = public_draft.description;
        self.weight_g = public_draft.weight_g;
        self.official_price_cents = public_draft.official_price_cents;
        self.official_price_currency = public_draft.official_price_currency;
        self.variants = public_draft.variants;
        self.specs = public_draft.specs;
        self.submitted_by_user_id = public_draft.submitted_by_user_id;

        self.source_key = normalize_required_text(
            std::mem::take(&mut self.source_key),
            160,
            "source_key",
            &mut errors,
        );
        self.source_name = normalize_required_text(
            std::mem::take(&mut self.source_name),
            80,
            "source_name",
            &mut errors,
        );
        self.source_url =
            normalize_optional_text(self.source_url.take(), 500, "source_url", &mut errors);
        self.source_license_note = normalize_optional_text(
            self.source_license_note.take(),
            240,
            "source_license_note",
            &mut errors,
        );
        self.import_batch_id = normalize_optional_text(
            self.import_batch_id.take(),
            128,
            "import_batch_id",
            &mut errors,
        );
        self.canonical_key =
            normalize_optional_text(self.canonical_key.take(), 160, "canonical_key", &mut errors);

        if self
            .source_rating_score
            .is_some_and(|score| !score.is_finite() || !(0.0..=10.0).contains(&score))
        {
            errors.push(FieldViolation::new(
                "source_rating_score",
                "must be between 0 and 10",
            ));
        }
        if self
            .source_rating_count
            .is_some_and(|count| !(0..=1_000_000).contains(&count))
        {
            errors.push(FieldViolation::new(
                "source_rating_count",
                "must be between 0 and 1000000",
            ));
        }
        if self
            .detail_score
            .is_some_and(|score| !(0..=10_000).contains(&score))
        {
            errors.push(FieldViolation::new(
                "detail_score",
                "must be between 0 and 10000",
            ));
        }

        for localization in &mut self.localizations {
            localization.validate_and_normalize(self.category, &mut errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

impl GearAtlasLocalizationDraft {
    /// Validates and normalizes a single admin-edited localization row.
    pub fn validate_and_normalize_for_category(
        &mut self,
        category: GearCategory,
    ) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        self.validate_and_normalize_with_prefix(
            category,
            &format!("localizations.{}", self.locale.as_str()),
            &mut errors,
        );
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }

    fn validate_and_normalize(&mut self, category: GearCategory, errors: &mut Vec<FieldViolation>) {
        self.validate_and_normalize_with_prefix(
            category,
            &format!("localizations.{}", self.locale.as_str()),
            errors,
        );
    }

    fn validate_and_normalize_with_prefix(
        &mut self,
        category: GearCategory,
        field_prefix: &str,
        errors: &mut Vec<FieldViolation>,
    ) {
        self.name = normalize_required_text(
            std::mem::take(&mut self.name),
            100,
            &format!("{field_prefix}.name"),
            errors,
        );
        self.description = normalize_optional_text(
            self.description.take(),
            100,
            &format!("{field_prefix}.description"),
            errors,
        );
        self.variants = normalize_variants(std::mem::take(&mut self.variants), errors);
        self.specs = normalize_specs(category, std::mem::take(&mut self.specs), errors);
        self.translation_status = normalize_optional_text(
            self.translation_status.take(),
            32,
            &format!("{field_prefix}.translation_status"),
            errors,
        );
        self.translation_provider = normalize_optional_text(
            self.translation_provider.take(),
            80,
            &format!("{field_prefix}.translation_provider"),
            errors,
        );
        self.translated_at = normalize_optional_text(
            self.translated_at.take(),
            64,
            &format!("{field_prefix}.translated_at"),
            errors,
        );
    }
}

/// Builds a public atlas draft from a personal gear item by copying only the
/// approved public snapshot fields.
pub fn draft_from_personal_gear(user_id: &str, item: &crate::gear::GearItem) -> GearAtlasDraft {
    GearAtlasDraft {
        category: item.category,
        name: item.name.clone(),
        brand: item.brand.clone(),
        model: item.model.clone(),
        description: item.description.clone(),
        weight_g: item.weight_g,
        official_price_cents: item.official_price_cents,
        official_price_currency: item.official_price_currency.clone(),
        variants: variant_from_personal_gear(item),
        specs: item.specs.clone(),
        source_type: GearAtlasSourceType::UserGear,
        submitted_by_user_id: user_id.to_owned(),
        source_user_gear_id: Some(item.id.clone()),
    }
}

fn variant_from_personal_gear(item: &crate::gear::GearItem) -> GearVariants {
    let Some(label) = item.selected_variant_label.as_deref() else {
        return GearVariants::new();
    };
    vec![GearVariant {
        key: item
            .selected_variant_key
            .clone()
            .unwrap_or_else(|| variant_key_from_label(label, 0)),
        label: label.to_owned(),
        official_price_cents: None,
        official_price_currency: None,
        weight_g: None,
    }]
}

/// Builds a compact user-visible summary of public fields changed before approval.
pub fn review_changes_between(
    before: &GearAtlasPublicSnapshot,
    after: &GearAtlasPublicSnapshot,
) -> GearAtlasReviewChanges {
    let mut changes = GearAtlasReviewChanges::new();
    push_change(
        &mut changes,
        "category",
        "分类",
        Some(before.category.label().to_owned()),
        Some(after.category.label().to_owned()),
    );
    push_change(
        &mut changes,
        "name",
        "名称",
        Some(before.name.clone()),
        Some(after.name.clone()),
    );
    push_change(
        &mut changes,
        "brand",
        "品牌",
        before.brand.clone(),
        after.brand.clone(),
    );
    push_change(
        &mut changes,
        "model",
        "型号",
        before.model.clone(),
        after.model.clone(),
    );
    push_change(
        &mut changes,
        "description",
        "描述",
        before.description.clone(),
        after.description.clone(),
    );
    push_change(
        &mut changes,
        "weight_g",
        "重量",
        before.weight_g.map(|value| format!("{value} g")),
        after.weight_g.map(|value| format!("{value} g")),
    );
    push_change(
        &mut changes,
        "official_price",
        "官方价格",
        price_summary(
            before.official_price_cents,
            before.official_price_currency.as_deref(),
        ),
        price_summary(
            after.official_price_cents,
            after.official_price_currency.as_deref(),
        ),
    );
    push_change(
        &mut changes,
        "variants",
        "可选尺寸",
        variants_summary(&before.variants),
        variants_summary(&after.variants),
    );

    let keys: BTreeSet<&String> = before.specs.keys().chain(after.specs.keys()).collect();
    for key in keys {
        push_change(
            &mut changes,
            &format!("specs.{key}"),
            &format!("分类参数 · {}", spec_review_label(key)),
            before.specs.get(key).cloned(),
            after.specs.get(key).cloned(),
        );
    }

    changes
}

fn push_change(
    changes: &mut GearAtlasReviewChanges,
    field: &str,
    label: &str,
    before: Option<String>,
    after: Option<String>,
) {
    if before == after {
        return;
    }
    changes.push(GearAtlasReviewChange {
        field: field.to_owned(),
        label: label.to_owned(),
        before,
        after,
    });
}

fn price_summary(cents: Option<i64>, currency: Option<&str>) -> Option<String> {
    let cents = cents?;
    let currency = currency.unwrap_or("CNY");
    let amount = format!("{:.2}", cents as f64 / 100.0);
    Some(match currency {
        "CNY" => format!("¥{amount}"),
        "USD" => format!("${amount}"),
        "EUR" => format!("€{amount}"),
        "JPY" => format!("¥{amount}"),
        "HKD" => format!("HK${amount}"),
        other => format!("{other} {amount}"),
    })
}

fn variants_summary(variants: &GearVariants) -> Option<String> {
    if variants.is_empty() {
        return None;
    }
    Some(
        variants
            .iter()
            .map(|variant| {
                let details = [
                    price_summary(
                        variant.official_price_cents,
                        variant.official_price_currency.as_deref(),
                    ),
                    variant.weight_g.map(|value| format!("{value} g")),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
                if details.is_empty() {
                    variant.label.clone()
                } else {
                    format!("{} · {}", variant.label, details.join(" · "))
                }
            })
            .collect::<Vec<_>>()
            .join("；"),
    )
}

fn spec_review_label(key: &str) -> String {
    match key {
        "accessories" => "附件",
        "back_length" => "背长",
        "battery_capacity" => "电池容量",
        "battery_type" => "电池类型",
        "beam_distance" => "照射距离",
        "breathability_rating" => "透湿指数",
        "capacity" => "容量",
        "certification" => "认证标准",
        "charging_port" => "充电接口",
        "days" => "适用天数",
        "expiry_date" => "有效期",
        "expiry_or_warranty_date" => "有效期/质保期",
        "fill_weight" => "填充重量",
        "filling" => "填充物",
        "fuel_type" => "燃料类型",
        "kit_size" => "套装规格",
        "layer" => "适用层级",
        "length" => "长度",
        "material" => "材质",
        "max_brightness" => "最大亮度",
        "net_content" => "净含量",
        "output_power" => "输出功率",
        "packed_size" => "收纳尺寸",
        "people_count" => "适用人数",
        "ports" => "接口类型",
        "power" => "功率",
        "quantity" => "数量",
        "rated_energy" => "额定能量",
        "recommended_load" => "推荐负重",
        "restock_threshold" => "补货阈值",
        "retirement_date" => "报废期限",
        "runtime" => "续航时间",
        "season" => "适用季节",
        "specification" => "规格",
        "storage_condition" => "储存条件",
        "strength" => "承重/强度",
        "support" => "缓震/支撑",
        "temperature_or_r_value" => "温标/R 值",
        "terrain" => "适用地形",
        "type" => "类型",
        "use_case" => "用途",
        "warmth_rating" => "保暖等级",
        "waterproof_packaging" => "防水包装",
        "waterproof_rating" => "防水等级",
        "working_temperature" => "工作温度",
        _ => key,
    }
    .to_owned()
}

fn translate_zh_to_en(value: &str) -> String {
    let mut output = value.to_owned();
    for (zh, en) in [
        ("户外", "Outdoor"),
        ("双肩包", "Backpack"),
        ("背包", "Backpack"),
        ("腰包", "Waist Pack"),
        ("帐篷", "Tent"),
        ("睡袋", "Sleeping Bag"),
        ("露营袋", "Bivy"),
        ("睡垫", "Sleeping Pad"),
        ("防潮垫", "Sleeping Pad"),
        ("头灯", "Headlamp"),
        ("手电", "Flashlight"),
        ("炉具", "Stove"),
        ("餐具", "Cookware"),
        ("水壶", "Water Bottle"),
        ("营地灯", "Lantern"),
        ("夹克", "Jacket"),
        ("冲锋衣", "Shell Jacket"),
        ("羽绒服", "Down Jacket"),
        ("登山鞋", "Hiking Boots"),
        ("徒步鞋", "Hiking Shoes"),
        ("越野跑鞋", "Trail Running Shoes"),
        ("登山杖", "Trekking Poles"),
        ("手表", "Watch"),
        ("安全带", "Harness"),
        ("头盔", "Helmet"),
        ("绳索", "Rope"),
        ("超轻", "Ultralight"),
        ("轻量", "Lightweight"),
        ("保温", "Insulated"),
        ("带水袋", "with Reservoir"),
        ("男女通用", "Unisex"),
        ("探路者", "Toread"),
    ] {
        output = output.replace(zh, en);
    }
    output
}

fn translate_en_to_zh(value: &str) -> String {
    let mut output = value.to_owned();
    for (en, zh) in [
        ("Sleeping Bags", "睡袋"),
        ("sleeping bags", "睡袋"),
        ("Sleeping Bag", "睡袋"),
        ("sleeping bag", "睡袋"),
        ("Backpacking Tents", "徒步帐篷"),
        ("backpacking tents", "徒步帐篷"),
        ("Backpacking Tent", "徒步帐篷"),
        ("backpacking tent", "徒步帐篷"),
        ("Bivy Sacks", "露营袋"),
        ("bivy sacks", "露营袋"),
        ("Bivy Sack", "露营袋"),
        ("bivy sack", "露营袋"),
        ("Bivies", "露营袋"),
        ("bivies", "露营袋"),
        ("Bivy", "露营袋"),
        ("bivy", "露营袋"),
        ("Bivvy", "露营袋"),
        ("bivvy", "露营袋"),
        ("Sleeping Pads", "睡垫"),
        ("sleeping pads", "睡垫"),
        ("Sleeping Pad", "睡垫"),
        ("sleeping pad", "睡垫"),
        ("Hydration Packs", "水袋背包"),
        ("hydration packs", "水袋背包"),
        ("Hydration Pack", "水袋背包"),
        ("hydration pack", "水袋背包"),
        ("Daypacks", "日用背包"),
        ("daypacks", "日用背包"),
        ("Daypack", "日用背包"),
        ("daypack", "日用背包"),
        ("Backpacks", "背包"),
        ("backpacks", "背包"),
        ("Backpack", "背包"),
        ("backpack", "背包"),
        ("Tents", "帐篷"),
        ("tents", "帐篷"),
        ("Tent", "帐篷"),
        ("tent", "帐篷"),
        ("Stoves", "炉具"),
        ("stoves", "炉具"),
        ("Stove", "炉具"),
        ("stove", "炉具"),
        ("Headlamps", "头灯"),
        ("headlamps", "头灯"),
        ("Headlamp", "头灯"),
        ("headlamp", "头灯"),
        ("Flashlights", "手电"),
        ("flashlights", "手电"),
        ("Flashlight", "手电"),
        ("flashlight", "手电"),
        ("Lanterns", "营地灯"),
        ("lanterns", "营地灯"),
        ("Lantern", "营地灯"),
        ("lantern", "营地灯"),
        ("Water Bottles", "水壶"),
        ("water bottles", "水壶"),
        ("Water Bottle", "水壶"),
        ("water bottle", "水壶"),
        ("Trekking Poles", "登山杖"),
        ("trekking poles", "登山杖"),
        ("Hiking Poles", "登山杖"),
        ("hiking poles", "登山杖"),
        ("Trail Running Shoes", "越野跑鞋"),
        ("trail running shoes", "越野跑鞋"),
        ("Hiking Boots", "登山鞋"),
        ("hiking boots", "登山鞋"),
        ("Hiking Shoes", "徒步鞋"),
        ("hiking shoes", "徒步鞋"),
        ("Down Jackets", "羽绒服"),
        ("down jackets", "羽绒服"),
        ("Down Jacket", "羽绒服"),
        ("down jacket", "羽绒服"),
        ("Shell Jackets", "冲锋衣"),
        ("shell jackets", "冲锋衣"),
        ("Shell Jacket", "冲锋衣"),
        ("shell jacket", "冲锋衣"),
        ("Jackets", "夹克"),
        ("jackets", "夹克"),
        ("Jacket", "夹克"),
        ("jacket", "夹克"),
        ("Harnesses", "安全带"),
        ("harnesses", "安全带"),
        ("Harness", "安全带"),
        ("harness", "安全带"),
        ("Helmets", "头盔"),
        ("helmets", "头盔"),
        ("Helmet", "头盔"),
        ("helmet", "头盔"),
        ("Ropes", "绳索"),
        ("ropes", "绳索"),
        ("Rope", "绳索"),
        ("rope", "绳索"),
        ("with Reservoir", "带水袋"),
        ("With Reservoir", "带水袋"),
        ("Reservoir", "水袋"),
        ("reservoir", "水袋"),
        ("Ultralight", "超轻"),
        ("ultralight", "超轻"),
        ("Lightweight", "轻量"),
        ("lightweight", "轻量"),
        ("Insulated", "保温"),
        ("insulated", "保温"),
        ("Degree", "度"),
        ("degree", "度"),
        ("Unisex", "男女通用"),
        ("unisex", "男女通用"),
    ] {
        output = output.replace(en, zh);
    }
    output
}

fn is_ascii_letter(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

fn normalize_official_price_currency(
    price_cents: Option<i64>,
    currency: Option<String>,
    errors: &mut Vec<FieldViolation>,
) -> Option<String> {
    let _price_cents = price_cents?;
    let normalized = currency
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "CNY".to_owned());
    if !SUPPORTED_CURRENCIES.contains(&normalized.as_str()) {
        errors.push(FieldViolation::new(
            "official_price_currency",
            "must be one of CNY, USD, EUR, JPY, HKD",
        ));
    }
    Some(normalized)
}

/// Returns the current UTC timestamp for atlas review transitions.
pub fn now_atlas_rfc3339() -> String {
    now_rfc3339()
}
