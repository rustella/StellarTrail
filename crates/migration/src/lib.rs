use sea_orm_migration::prelude::*;

mod m20260516_000001_create_users_sessions;
mod m20260516_000002_create_user_gear_items;
mod m20260516_000003_add_password_auth;
mod m20260516_000004_create_captcha_challenges;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260516_000001_create_users_sessions::Migration),
            Box::new(m20260516_000002_create_user_gear_items::Migration),
            Box::new(m20260516_000003_add_password_auth::Migration),
            Box::new(m20260516_000004_create_captcha_challenges::Migration),
        ]
    }
}
