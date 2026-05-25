//! Adds locale-scoped alias JSON to existing knot localizations.
//!
//! Aliases are localized public content, so they belong beside the title,
//! summary, description, and steps for the same `(knot_id, locale)` row.

use sea_orm_migration::prelude::*;

/// Adds `knot_localizations.aliases_json` as a non-null JSON array string.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Adds the alias JSON column with an empty-array default for existing rows.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE knot_localizations ADD COLUMN aliases_json TEXT NOT NULL DEFAULT '[]'",
            )
            .await?;
        Ok(())
    }

    /// Removes only the alias JSON column introduced by this migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE knot_localizations DROP COLUMN aliases_json")
            .await?;
        Ok(())
    }
}
