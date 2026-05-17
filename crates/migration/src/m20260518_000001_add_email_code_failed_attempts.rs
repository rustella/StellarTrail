//! Adds failed-attempt tracking to email verification codes.
//!
//! Login and password-reset codes are intentionally short because users type
//! them manually. Tracking failed attempts lets the API invalidate a code after
//! repeated guesses while preserving the existing one-time consumption behavior
//! for successful submissions.

use sea_orm_migration::prelude::*;

/// SeaORM migration that extends email verification codes with a brute-force guard.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Adds a per-code failed-attempt counter with a zero default for existing rows.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE email_verification_codes ADD COLUMN failed_attempts INTEGER NOT NULL DEFAULT 0",
            )
            .await?;
        Ok(())
    }

    /// Rollback leaves the additive column in place on SQLite-compatible deployments.
    ///
    /// The project already treats SQLite down migrations for additive auth columns
    /// as best-effort because SQLite cannot drop columns without rebuilding the
    /// table. The extra counter is ignored by older code and is harmless.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
