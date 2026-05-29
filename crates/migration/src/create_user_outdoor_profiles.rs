//! Creates account-level outdoor profile defaults for trip planning.

use sea_orm_migration::prelude::*;

/// Migration type invoked by SeaORM for outdoor profile schema changes.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "create_user_outdoor_profiles"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates a single-row-per-user outdoor profile table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS user_outdoor_profiles (
                user_id TEXT PRIMARY KEY REFERENCES users(id),
                outdoor_id TEXT NULL,
                real_name TEXT NULL,
                gender TEXT NULL,
                height_cm INTEGER NULL,
                phone TEXT NULL,
                emergency_contact TEXT NULL,
                emergency_phone TEXT NULL,
                blood_type TEXT NULL,
                medical_history TEXT NULL,
                allergy_history TEXT NULL,
                insurance_policy_no TEXT NULL,
                experience_note TEXT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"#,
        )
        .await?;
        Ok(())
    }

    /// Drops the outdoor profile table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS user_outdoor_profiles")
            .await?;
        Ok(())
    }
}
