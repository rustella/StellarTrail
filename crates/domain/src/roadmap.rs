//! Roadmap domain model and validation for DB-backed public product planning.
//!
//! Roadmap entries are product-planning content, not feature implementations.
//! Keeping the status and category vocabularies explicit prevents clients and
//! administrators from creating arbitrary values that later become hard to
//! render consistently.

use serde::{Deserialize, Serialize};

use crate::validation::{
    FieldViolation, ValidationError, normalize_optional_text, normalize_required_text,
};

/// Client keys supported by roadmap entries.
pub const ROADMAP_CLIENT_KEYS: &[&str] = &["wechat_miniprogram", "web", "android", "ios", "macos"];

/// Public lifecycle states for a roadmap item.
pub const ROADMAP_STATUSES: &[&str] = &["planned", "designing", "building", "preview", "shipped"];

/// Product categories used for grouping roadmap items.
pub const ROADMAP_CATEGORIES: &[&str] =
    &["gear", "skills", "routes", "offline", "safety", "community"];

/// Writable roadmap item fields accepted by administrator APIs.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoadmapItemDraft {
    pub client_key: String,
    pub title: String,
    pub summary: String,
    pub details: Option<String>,
    pub category: String,
    pub status: String,
    pub priority: i32,
    pub sort_order: i32,
    pub is_published: bool,
}

impl RoadmapItemDraft {
    /// Validates and trims administrator roadmap content before persistence.
    pub fn validate_and_normalize(&mut self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        self.client_key = normalize_enum(
            std::mem::take(&mut self.client_key),
            ROADMAP_CLIENT_KEYS,
            "client_key",
            &mut errors,
        );
        self.title =
            normalize_required_text(std::mem::take(&mut self.title), 120, "title", &mut errors);
        self.summary = normalize_required_text(
            std::mem::take(&mut self.summary),
            240,
            "summary",
            &mut errors,
        );
        self.details = normalize_optional_text(self.details.take(), 2000, "details", &mut errors);
        self.category = normalize_enum(
            std::mem::take(&mut self.category),
            ROADMAP_CATEGORIES,
            "category",
            &mut errors,
        );
        self.status = normalize_enum(
            std::mem::take(&mut self.status),
            ROADMAP_STATUSES,
            "status",
            &mut errors,
        );
        if !(0..=100).contains(&self.priority) {
            errors.push(FieldViolation::new("priority", "must be between 0 and 100"));
        }
        if !(-100_000..=100_000).contains(&self.sort_order) {
            errors.push(FieldViolation::new(
                "sort_order",
                "must be between -100000 and 100000",
            ));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::new(errors))
        }
    }
}

/// Returns true when the provided client key is supported by Roadmap APIs.
pub fn is_valid_roadmap_client_key(value: &str) -> bool {
    ROADMAP_CLIENT_KEYS.contains(&value)
}

/// Returns true when the provided public item status is supported.
pub fn is_valid_roadmap_status(value: &str) -> bool {
    ROADMAP_STATUSES.contains(&value)
}

fn normalize_enum(
    value: String,
    allowed: &[&str],
    field: &'static str,
    errors: &mut Vec<FieldViolation>,
) -> String {
    let value = value.trim().to_ascii_lowercase();
    if !allowed.contains(&value.as_str()) {
        errors.push(FieldViolation::new(field, "is not supported"));
    }
    value
}
