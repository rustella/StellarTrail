//! SeaORM migration registry for the StellarTrail schema.
//!
//! Migrations are registered in chronological order so local SQLite tests,
//! containerized PostgreSQL integration tests, and production deployments all
//! apply the same schema sequence. New migrations should be appended to this
//! list instead of inserted between already-released entries.

use sea_orm_migration::prelude::*;

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
        ]
    }
}
