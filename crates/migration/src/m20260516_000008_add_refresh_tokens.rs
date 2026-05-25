//! Adds refresh-token persistence metadata to the `sessions` table.
//!
//! StellarTrail uses opaque bearer tokens rather than JWTs. The API returns the
//! plaintext refresh token to the client only in login or refresh responses, and
//! the database stores only a hash of that token. This migration adds the hash,
//! expiry, and rotation timestamp columns needed to renew short-lived access
//! tokens while keeping existing session rows valid during rollout.

use sea_orm_migration::prelude::*;

/// SeaORM migration that extends existing sessions with refresh-token state.
///
/// The migration is intentionally additive: every new column is nullable so
/// sessions created before deployment can still authenticate with their access
/// tokens, while only newly issued sessions are eligible for refresh-token
/// renewal.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Applies the additive refresh-token schema changes.
    ///
    /// The columns are stored as text to match the existing timestamp and token
    /// hash storage style used by the session table. Keeping the values nullable
    /// avoids forcing an immediate re-login for users who only have legacy
    /// access-token sessions.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Store only the refresh-token digest; the plaintext token is returned
        // to clients once and is never persisted server-side.
        db.execute_unprepared("ALTER TABLE sessions ADD COLUMN refresh_token_hash TEXT NULL")
            .await?;
        // Track refresh-token expiry separately from the shorter access-token
        // expiry so clients can renew access without extending a revoked session.
        db.execute_unprepared("ALTER TABLE sessions ADD COLUMN refresh_expires_at TEXT NULL")
            .await?;
        // Record the most recent successful rotation for auditing and future
        // cleanup jobs without changing current authentication behavior.
        db.execute_unprepared("ALTER TABLE sessions ADD COLUMN refreshed_at TEXT NULL")
            .await?;
        // Enforce deterministic hash lookups and prevent accidental duplicate
        // refresh-token hashes while still allowing legacy NULL rows.
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_refresh_token_hash ON sessions(refresh_token_hash)",
        )
        .await?;
        Ok(())
    }

    /// Removes the refresh-token lookup index during rollback.
    ///
    /// SQLite cannot safely drop columns without rebuilding the table, so the
    /// down migration only removes the index. The leftover nullable columns are
    /// harmless and keep rollback behavior consistent across local SQLite tests.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Drop the only schema object that SQLite can remove safely here.
        db.execute_unprepared("DROP INDEX IF EXISTS idx_sessions_refresh_token_hash")
            .await?;
        Ok(())
    }
}
