//! SeaORM migration registry for the StellarTrail schema.
//!
//! Migrations are registered in the order required to build a fresh schema.
//! Schema-only patch migrations may be folded into their base tables when the
//! project intentionally drops compatibility with already-applied histories.

use sea_orm_migration::prelude::*;

mod add_public_content_localizations;
mod clear_knots3d_section_heading_steps;
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
mod create_upload_images;
mod create_user_disclaimer_acceptances;
mod create_user_feedback;
mod create_user_gear_items;
mod create_user_knot_favorites;
mod create_users_sessions;
mod sanitize_knot_risk_copy;

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
            Box::new(create_captcha_challenges::Migration),
            Box::new(create_upload_images::Migration),
            Box::new(create_user_feedback::Migration),
            Box::new(create_knots_content::Migration),
            Box::new(create_media_resources::Migration),
            Box::new(create_gear_templates::Migration),
            Box::new(clear_knots3d_section_heading_steps::Migration),
            Box::new(create_api_usage_daily::Migration),
            Box::new(create_gear_atlas_items::Migration),
            Box::new(create_admin_roles::Migration),
            Box::new(add_public_content_localizations::Migration),
            Box::new(create_client_versions::Migration),
            Box::new(create_gear_packing_lists::Migration),
            Box::new(create_user_knot_favorites::Migration),
            Box::new(create_user_disclaimer_acceptances::Migration),
            Box::new(create_roadmap::Migration),
            Box::new(sanitize_knot_risk_copy::Migration),
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
            "m20260516_000004_create_captcha_challenges",
            "m20260516_000005_create_upload_images",
            "m20260516_000006_create_user_feedback",
            "m20260516_000007_create_knots_content",
            "m20260517_000006_create_media_resources",
            "m20260517_000007_create_gear_templates",
            "m20260518_000002_clear_knots3d_section_heading_steps",
            "m20260518_000003_create_api_usage_daily",
            "m20260518_000005_create_gear_atlas_items",
            "m20260518_000006_create_admin_roles",
            "m20260521_000001_add_public_content_localizations",
            "create_client_versions",
            "m20260524_000001_create_gear_packing_lists",
            "m20260524_000002_create_user_knot_favorites",
            "m20260524_000003_create_user_disclaimer_acceptances",
            "m20260524_000005_create_roadmap",
            "m20260524_000006_sanitize_knot_risk_copy",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();

        assert_eq!(names, expected_names);
    }
}
