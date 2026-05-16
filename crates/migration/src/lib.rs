//! Database migration crate entrypoint registering all StellarTrail schema migrations in order.

use sea_orm_migration::prelude::*;

mod m20260516_000001_create_users_sessions;
mod m20260516_000002_create_user_gear_items;
mod m20260516_000003_add_password_auth;
mod m20260516_000004_create_captcha_challenges;
mod m20260516_000005_create_upload_images;
mod m20260516_000006_create_user_feedback;

/// SeaORM migrator implementation that runs all schema migrations in registration order.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    /// Runs the `migrations` server-side flow while preserving input validation, error propagation, and state invariants.
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Migration order is the schema evolution order, so new migrations must be appended to the end of the list.
        vec![
            Box::new(m20260516_000001_create_users_sessions::Migration),
            Box::new(m20260516_000002_create_user_gear_items::Migration),
            Box::new(m20260516_000003_add_password_auth::Migration),
            Box::new(m20260516_000004_create_captcha_challenges::Migration),
            Box::new(m20260516_000005_create_upload_images::Migration),
            Box::new(m20260516_000006_create_user_feedback::Migration),
        ]
    }
}
