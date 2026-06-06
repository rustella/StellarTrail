//! Repository aggregation module that re-exports authentication, gear, feedback, upload image, and knot persistence objects.

pub mod admin_role_repository;
pub mod api_usage_repository;
pub mod app_content_page_repository;
pub mod auth_repository;
pub mod client_version_repository;
pub mod disclaimer_acceptance_repository;
pub mod feedback_repository;
pub mod gear_atlas_repository;
pub mod gear_packing_repository;
pub mod gear_repository;
pub mod gear_template_repository;
pub mod knot_repository;
pub mod media_resource_repository;
pub mod outdoor_profile_repository;
pub mod roadmap_repository;
pub mod skill_favorite_repository;
pub mod trip_repository;
pub mod upload_image_repository;

pub use admin_role_repository::{
    AdminRoleRecord, AdminRoleRepository, AdminTargetUser, GrantAdminResult,
};
pub use api_usage_repository::{
    ApiUsageIncrement, ApiUsageQuery, ApiUsageRecord, ApiUsageRepository,
};
pub use app_content_page_repository::{AppContentPageRecord, AppContentPageRepository};
pub use auth_repository::{AuthRepository, UserRecord, hash_token};
pub use client_version_repository::{
    ClientVersionDraft, ClientVersionRecord, ClientVersionRepository, ListClientVersionsOptions,
};
pub use disclaimer_acceptance_repository::{
    DisclaimerAcceptanceDraft, DisclaimerAcceptanceRecord, DisclaimerAcceptanceRepository,
};
pub use feedback_repository::{
    AdminFeedbackRecord, FeedbackAuthorRecord, FeedbackRecord, FeedbackRepository,
    ListAdminFeedbackOptions,
};
pub use gear_atlas_repository::{
    GearAtlasExternalImportAction, GearAtlasExternalImportResult, GearAtlasRepository,
    ListGearAtlasAdminOptions, ListGearAtlasOptions,
};
pub use gear_packing_repository::{
    AddGearPackingItemsResult, GearPackingRepository, ListGearPackingListsOptions,
};
pub use gear_repository::{GearRepository, ListGearOptions};
pub use gear_template_repository::GearTemplateRepository;
pub use knot_repository::KnotRepository;
pub use media_resource_repository::{
    KnotMediaLinkDraft, MediaResourceDraft, MediaResourceRecord, MediaResourceRepository,
};
pub use outdoor_profile_repository::OutdoorProfileRepository;
pub use roadmap_repository::{
    ListRoadmapOptions, RoadmapItemRecord, RoadmapListEntry, RoadmapRepository,
};
pub use skill_favorite_repository::{
    KnotFavoriteListEntry, KnotFavoriteStatus, SkillFavoriteCounts, SkillFavoriteRepository,
};
pub use trip_repository::{
    KIND_BUDGET_ITEM, KIND_FOOD_ITEM, KIND_FOOD_MEAL, KIND_FOOD_SUPPLY, KIND_GOAL_ITEM,
    KIND_ITINERARY_DAY, KIND_MEDICAL_ITEM, KIND_PERSONAL_GEAR, KIND_RESCUE_CONTACT,
    KIND_ROUTE_SEGMENT, KIND_SAFETY_RISK, KIND_SEGMENT_ASSIGNMENT, KIND_SHARED_GEAR,
    KIND_TIME_SLOT, ListTripsOptions, TripHighlightStatus, TripHomeHighlight, TripRepository,
    TripRepositoryError,
};
pub use upload_image_repository::{UploadImageDraft, UploadImageRecord, UploadImageRepository};

use sea_orm::{DatabaseBackend, Statement, Value};

pub(crate) fn statement(
    backend: DatabaseBackend,
    sql: impl Into<String>,
    values: Vec<Value>,
) -> Statement {
    let sql = sql.into();
    let sql = if matches!(backend, DatabaseBackend::Postgres) {
        postgres_placeholders(&sql)
    } else {
        sql
    };
    Statement::from_sql_and_values(backend, sql, values)
}

fn postgres_placeholders(sql: &str) -> String {
    let mut converted = String::with_capacity(sql.len());
    let mut index = 1;
    let mut in_single_quote = false;
    let mut chars = sql.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\'' {
            converted.push(ch);
            if in_single_quote && chars.peek() == Some(&'\'') {
                converted.push(chars.next().expect("peeked escaped quote"));
            } else {
                in_single_quote = !in_single_quote;
            }
            continue;
        }

        if ch == '?' && !in_single_quote {
            converted.push('$');
            converted.push_str(&index.to_string());
            index += 1;
        } else {
            converted.push(ch);
        }
    }

    converted
}

#[cfg(test)]
mod tests {
    use super::postgres_placeholders;

    #[test]
    fn converts_question_mark_placeholders_for_postgres() {
        assert_eq!(
            postgres_placeholders("SELECT * FROM users WHERE id = ? AND email = ?"),
            "SELECT * FROM users WHERE id = $1 AND email = $2",
        );
    }

    #[test]
    fn leaves_question_marks_inside_sql_strings_unchanged() {
        assert_eq!(
            postgres_placeholders(
                "SELECT '?' AS literal, name FROM users WHERE id = ? AND note = 'it''s ?'"
            ),
            "SELECT '?' AS literal, name FROM users WHERE id = $1 AND note = 'it''s ?'",
        );
    }
}
