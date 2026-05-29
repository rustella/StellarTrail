//! Adds birth date to reusable outdoor profiles.

use sea_orm_migration::prelude::*;

/// Migration that stores a user's birth date for age display.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260527_000002_add_outdoor_profile_birth_date"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Adds an optional ISO date string; age is derived by clients.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE user_outdoor_profiles ADD COLUMN birth_date TEXT NULL")
            .await?;
        Ok(())
    }

    /// Keeps the column for compatibility on databases that cannot drop columns safely.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
