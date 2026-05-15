use serde::{Deserialize, Serialize};
use stellartrail_domain::gear::{
    GearCategory, GearDraft, GearItem, GearShareStatus, GearSort, GearStatus, GearTab,
};

#[derive(Debug, Deserialize)]
pub struct ListGearQuery {
    #[serde(default)]
    pub tab: GearTab,
    pub category: Option<GearCategory>,
    pub status: Option<GearStatus>,
    pub q: Option<String>,
    #[serde(default)]
    pub sort: GearSort,
    pub limit: Option<u64>,
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GearStatsQuery {
    #[serde(default)]
    pub tab: GearTab,
}

#[derive(Debug, Deserialize)]
pub struct GearExportQuery {
    #[serde(default)]
    pub tab: GearTab,
    #[serde(default = "default_csv_format")]
    pub format: String,
}

fn default_csv_format() -> String {
    "csv".to_owned()
}

#[derive(Debug, Deserialize)]
pub struct CreateGearRequest {
    pub category: GearCategory,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub color: Option<String>,
    pub material: Option<String>,
    pub capacity: Option<String>,
    pub size: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub warmth_index: Option<String>,
    pub waterproof_index: Option<String>,
    pub purchase_date: Option<String>,
    pub purchase_price_cents: Option<i64>,
    pub expiry_or_warranty_date: Option<String>,
    pub purchase_location: Option<String>,
    pub status: Option<GearStatus>,
    pub storage_location: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub share_enabled: bool,
    pub notes: Option<String>,
}

impl CreateGearRequest {
    pub fn into_draft(self) -> GearDraft {
        GearDraft {
            category: self.category,
            name: self.name,
            brand: self.brand,
            model: self.model,
            color: self.color,
            material: self.material,
            capacity: self.capacity,
            size: self.size,
            description: self.description,
            weight_g: self.weight_g,
            warmth_index: self.warmth_index,
            waterproof_index: self.waterproof_index,
            purchase_date: self.purchase_date,
            purchase_price_cents: self.purchase_price_cents,
            expiry_or_warranty_date: self.expiry_or_warranty_date,
            purchase_location: self.purchase_location,
            status: self.status.unwrap_or_default(),
            storage_location: self.storage_location,
            tags: self.tags,
            share_enabled: self.share_enabled,
            share_status: GearShareStatus::NotShared,
            notes: self.notes,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct UpdateGearRequest {
    pub category: Option<GearCategory>,
    pub name: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub color: Option<String>,
    pub material: Option<String>,
    pub capacity: Option<String>,
    pub size: Option<String>,
    pub description: Option<String>,
    pub weight_g: Option<i32>,
    pub warmth_index: Option<String>,
    pub waterproof_index: Option<String>,
    pub purchase_date: Option<String>,
    pub purchase_price_cents: Option<i64>,
    pub expiry_or_warranty_date: Option<String>,
    pub purchase_location: Option<String>,
    pub status: Option<GearStatus>,
    pub storage_location: Option<String>,
    pub tags: Option<Vec<String>>,
    pub share_enabled: Option<bool>,
    pub notes: Option<String>,
}

impl UpdateGearRequest {
    pub fn merge_into(self, existing: &GearItem) -> GearDraft {
        GearDraft {
            category: self.category.unwrap_or(existing.category),
            name: self.name.unwrap_or_else(|| existing.name.clone()),
            brand: self.brand.or_else(|| existing.brand.clone()),
            model: self.model.or_else(|| existing.model.clone()),
            color: self.color.or_else(|| existing.color.clone()),
            material: self.material.or_else(|| existing.material.clone()),
            capacity: self.capacity.or_else(|| existing.capacity.clone()),
            size: self.size.or_else(|| existing.size.clone()),
            description: self.description.or_else(|| existing.description.clone()),
            weight_g: self.weight_g.or(existing.weight_g),
            warmth_index: self.warmth_index.or_else(|| existing.warmth_index.clone()),
            waterproof_index: self
                .waterproof_index
                .or_else(|| existing.waterproof_index.clone()),
            purchase_date: self
                .purchase_date
                .or_else(|| existing.purchase_date.clone()),
            purchase_price_cents: self.purchase_price_cents.or(existing.purchase_price_cents),
            expiry_or_warranty_date: self
                .expiry_or_warranty_date
                .or_else(|| existing.expiry_or_warranty_date.clone()),
            purchase_location: self
                .purchase_location
                .or_else(|| existing.purchase_location.clone()),
            status: self.status.unwrap_or(existing.status),
            storage_location: self
                .storage_location
                .or_else(|| existing.storage_location.clone()),
            tags: self.tags.unwrap_or_else(|| existing.tags.clone()),
            share_enabled: self.share_enabled.unwrap_or(existing.share_enabled),
            share_status: existing.share_status,
            notes: self.notes.or_else(|| existing.notes.clone()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct GearSummaryResponse {
    pub id: String,
    pub category: GearCategory,
    pub category_label: String,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub status: GearStatus,
    pub status_label: String,
    pub weight_g: Option<i32>,
    pub purchase_price_cents: Option<i64>,
    pub purchase_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<&GearItem> for GearSummaryResponse {
    fn from(item: &GearItem) -> Self {
        Self {
            id: item.id.clone(),
            category: item.category,
            category_label: item.category.label().to_owned(),
            name: item.name.clone(),
            brand: item.brand.clone(),
            model: item.model.clone(),
            status: item.status,
            status_label: item.status.label().to_owned(),
            weight_g: item.weight_g,
            purchase_price_cents: item.purchase_price_cents,
            purchase_date: item.purchase_date.clone(),
            created_at: item.created_at.clone(),
            updated_at: item.updated_at.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ListGearResponse {
    pub items: Vec<GearSummaryResponse>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GearCategoryFilterResponse {
    pub id: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct GearCategoriesResponse {
    pub items: Vec<GearCategoryFilterResponse>,
}

#[derive(Debug, Deserialize)]
pub struct ImportGearsRequest {
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub items: Vec<CreateGearRequest>,
}

#[derive(Debug, Serialize)]
pub struct ImportGearsResponse {
    pub created_count: usize,
    pub updated_count: usize,
    pub failed_count: usize,
    pub errors: Vec<ImportGearError>,
}

#[derive(Debug, Serialize)]
pub struct ImportGearError {
    pub row: usize,
    pub field: String,
    pub message: String,
}
