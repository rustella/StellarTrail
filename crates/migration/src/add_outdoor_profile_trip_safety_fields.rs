//! Adds emergency, health handling, diet, and insurance contact fields.

use sea_orm_migration::prelude::*;

/// Migration that extends reusable outdoor profiles for trip member imports.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "add_outdoor_profile_trip_safety_fields"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Adds optional fields used by real trip member cards.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "ALTER TABLE user_outdoor_profiles ADD COLUMN emergency_contact_relationship TEXT NULL",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE user_outdoor_profiles ADD COLUMN medical_response_note TEXT NULL",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE user_outdoor_profiles ADD COLUMN diet_preference TEXT NULL",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE user_outdoor_profiles ADD COLUMN insurance_company_phone TEXT NULL",
        )
        .await?;
        Ok(())
    }

    /// Keeps added columns for compatibility on databases that cannot drop columns safely.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
