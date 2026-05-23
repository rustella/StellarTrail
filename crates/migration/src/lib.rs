//! SeaORM migration registry for the StellarTrail schema.
//!
//! Migrations are registered in chronological order so local SQLite tests,
//! containerized PostgreSQL integration tests, and production deployments all
//! apply the same schema sequence. New migrations should be appended to this
//! list instead of inserted between already-released entries.

use sea_orm_migration::prelude::*;

mod create_client_versions;
mod m20260516_000001_create_users_sessions;
mod m20260516_000002_create_user_gear_items;
mod m20260516_000003_add_password_auth;
mod m20260516_000004_create_captcha_challenges;
mod m20260516_000005_create_upload_images;
mod m20260516_000006_create_user_feedback;
mod m20260516_000007_create_knots_content;
mod m20260516_000008_add_refresh_tokens;
mod m20260517_000006_create_media_resources;
mod m20260517_000007_create_gear_templates;
mod m20260518_000001_add_email_code_failed_attempts;
mod m20260518_000002_clear_knots3d_section_heading_steps;
mod m20260518_000003_create_api_usage_daily;
mod m20260518_000004_add_gear_specs_prices;
mod m20260518_000005_create_gear_atlas_items;
mod m20260518_000006_create_admin_roles;
mod m20260521_000001_add_public_content_localizations;
mod m20260521_000002_add_gear_atlas_import_metadata;
mod m20260522_000001_add_gear_variants;
mod m20260523_000001_add_gear_atlas_review_snapshots;
mod m20260523_000002_remove_knot_difficulty;
mod m20260523_000003_add_upload_images_user_purpose_index;
mod m20260523_000004_add_soft_delete_flags;
mod m20260524_000001_create_gear_packing_lists;
mod m20260524_000002_create_user_knot_favorites;
mod m20260524_000004_add_gear_quantities;
mod m20260524_000005_create_roadmap;

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
            Box::new(m20260516_000001_create_users_sessions::Migration),
            Box::new(m20260516_000002_create_user_gear_items::Migration),
            Box::new(m20260516_000003_add_password_auth::Migration),
            Box::new(m20260516_000004_create_captcha_challenges::Migration),
            Box::new(m20260516_000005_create_upload_images::Migration),
            Box::new(m20260516_000006_create_user_feedback::Migration),
            Box::new(m20260516_000007_create_knots_content::Migration),
            Box::new(m20260516_000008_add_refresh_tokens::Migration),
            Box::new(m20260517_000006_create_media_resources::Migration),
            Box::new(m20260517_000007_create_gear_templates::Migration),
            Box::new(m20260518_000001_add_email_code_failed_attempts::Migration),
            Box::new(m20260518_000002_clear_knots3d_section_heading_steps::Migration),
            Box::new(m20260518_000003_create_api_usage_daily::Migration),
            Box::new(m20260518_000004_add_gear_specs_prices::Migration),
            Box::new(m20260518_000005_create_gear_atlas_items::Migration),
            Box::new(m20260518_000006_create_admin_roles::Migration),
            Box::new(m20260521_000001_add_public_content_localizations::Migration),
            Box::new(m20260521_000002_add_gear_atlas_import_metadata::Migration),
            Box::new(m20260522_000001_add_gear_variants::Migration),
            Box::new(m20260523_000001_add_gear_atlas_review_snapshots::Migration),
            Box::new(m20260523_000002_remove_knot_difficulty::Migration),
            Box::new(m20260523_000003_add_upload_images_user_purpose_index::Migration),
            Box::new(create_client_versions::Migration),
            Box::new(m20260523_000004_add_soft_delete_flags::Migration),
            Box::new(m20260524_000001_create_gear_packing_lists::Migration),
            Box::new(m20260524_000004_add_gear_quantities::Migration),
            Box::new(m20260524_000005_create_roadmap::Migration),
            Box::new(m20260524_000002_create_user_knot_favorites::Migration),
        ]
    }
}
