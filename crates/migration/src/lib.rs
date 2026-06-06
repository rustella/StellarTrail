//! SeaORM migration registry for the StellarTrail schema.
//!
//! Migrations are registered in the order required to build a fresh schema.
//! Schema-only changes must be folded into the corresponding initialization
//! migrations; this crate intentionally avoids new standalone schema patches.

use sea_orm_migration::prelude::*;

mod add_client_version_commit_hash;
mod add_outdoor_profile_birth_date;
mod add_outdoor_profile_trip_safety_fields;
mod add_public_content_localizations;
mod clear_knots3d_section_heading_steps;
mod compat_folded_migrations;
mod create_admin_roles;
mod create_api_usage_daily;
mod create_captcha_challenges;
mod create_client_versions;
mod create_gear_atlas_items;
mod create_gear_packing_lists;
mod create_gear_templates;
mod create_knots_content;
mod create_media_resources;
mod create_roadmap;
mod create_shared_gear_demand_templates;
mod create_trips;
mod create_upload_images;
mod create_user_disclaimer_acceptances;
mod create_user_feedback;
mod create_user_gear_items;
mod create_user_knot_favorites;
mod create_user_outdoor_profiles;
mod create_users_sessions;
mod ensure_users_phone_fields;
mod sanitize_knot_risk_copy;
mod update_shared_gear_demand_templates;

/// Concrete SeaORM migrator used by the API server and test suites.
///
/// Keeping the registration point centralized makes it obvious which schema
/// revisions are part of a release and prevents tests from accidentally running
/// a different migration set than the API binary.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    /// Returns every schema migration in the exact order it must be applied.
    ///
    /// The vector order is part of the database contract: later migrations may
    /// depend on columns, indexes, or tables created by earlier entries.
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Append new migrations at the end to preserve deployed revision order.
        vec![
            Box::new(create_users_sessions::Migration),
            Box::new(create_user_gear_items::Migration),
            Box::new(compat_folded_migrations::add_password_auth::Migration),
            Box::new(create_captcha_challenges::Migration),
            Box::new(create_upload_images::Migration),
            Box::new(create_user_feedback::Migration),
            Box::new(create_knots_content::Migration),
            Box::new(compat_folded_migrations::add_refresh_tokens::Migration),
            Box::new(create_media_resources::Migration),
            Box::new(create_gear_templates::Migration),
            Box::new(compat_folded_migrations::add_email_code_failed_attempts::Migration),
            Box::new(clear_knots3d_section_heading_steps::Migration),
            Box::new(create_api_usage_daily::Migration),
            Box::new(compat_folded_migrations::add_gear_specs_prices::Migration),
            Box::new(create_gear_atlas_items::Migration),
            Box::new(create_admin_roles::Migration),
            Box::new(add_public_content_localizations::Migration),
            Box::new(compat_folded_migrations::add_gear_atlas_import_metadata::Migration),
            Box::new(compat_folded_migrations::add_gear_variants::Migration),
            Box::new(compat_folded_migrations::add_gear_atlas_review_snapshots::Migration),
            Box::new(compat_folded_migrations::remove_knot_difficulty::Migration),
            Box::new(compat_folded_migrations::add_upload_images_user_purpose_index::Migration),
            Box::new(compat_folded_migrations::add_soft_delete_flags::Migration),
            Box::new(create_client_versions::Migration),
            Box::new(create_gear_packing_lists::Migration),
            Box::new(create_user_knot_favorites::Migration),
            Box::new(create_user_disclaimer_acceptances::Migration),
            Box::new(compat_folded_migrations::add_gear_quantities::Migration),
            Box::new(create_roadmap::Migration),
            Box::new(sanitize_knot_risk_copy::Migration),
            Box::new(compat_folded_migrations::add_knot_localization_aliases::Migration),
            Box::new(compat_folded_migrations::create_team_trip_plans::Migration),
            Box::new(create_trips::Migration),
            Box::new(create_shared_gear_demand_templates::Migration),
            Box::new(create_user_outdoor_profiles::Migration),
            Box::new(add_outdoor_profile_birth_date::Migration),
            Box::new(add_outdoor_profile_trip_safety_fields::Migration),
            Box::new(update_shared_gear_demand_templates::Migration),
            Box::new(add_client_version_commit_hash::Migration),
            Box::new(ensure_users_phone_fields::Migration),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_names_match_fresh_schema_history() {
        let names = Migrator::migrations()
            .into_iter()
            .map(|migration| migration.name().to_owned())
            .collect::<Vec<_>>();
        let expected_names = [
            "m20260516_000001_create_users_sessions",
            "m20260516_000002_create_user_gear_items",
            "m20260516_000003_add_password_auth",
            "m20260516_000004_create_captcha_challenges",
            "m20260516_000005_create_upload_images",
            "m20260516_000006_create_user_feedback",
            "m20260516_000007_create_knots_content",
            "m20260516_000008_add_refresh_tokens",
            "m20260517_000006_create_media_resources",
            "m20260517_000007_create_gear_templates",
            "m20260518_000001_add_email_code_failed_attempts",
            "m20260518_000002_clear_knots3d_section_heading_steps",
            "m20260518_000003_create_api_usage_daily",
            "m20260518_000004_add_gear_specs_prices",
            "m20260518_000005_create_gear_atlas_items",
            "m20260518_000006_create_admin_roles",
            "m20260521_000001_add_public_content_localizations",
            "m20260521_000002_add_gear_atlas_import_metadata",
            "m20260522_000001_add_gear_variants",
            "m20260523_000001_add_gear_atlas_review_snapshots",
            "m20260523_000002_remove_knot_difficulty",
            "m20260523_000003_add_upload_images_user_purpose_index",
            "m20260523_000004_add_soft_delete_flags",
            "create_client_versions",
            "m20260524_000001_create_gear_packing_lists",
            "m20260524_000002_create_user_knot_favorites",
            "m20260524_000003_create_user_disclaimer_acceptances",
            "m20260524_000004_add_gear_quantities",
            "m20260524_000005_create_roadmap",
            "m20260524_000006_sanitize_knot_risk_copy",
            "m20260525_000001_add_knot_localization_aliases",
            "create_team_trip_plans",
            "create_trips",
            "create_shared_gear_demand_templates",
            "create_user_outdoor_profiles",
            "m20260527_000002_add_outdoor_profile_birth_date",
            "add_outdoor_profile_trip_safety_fields",
            "update_shared_gear_demand_templates",
            "add_client_version_commit_hash",
            "m20260606_000001_ensure_users_phone_fields",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();

        assert_eq!(names, expected_names);
    }
}
