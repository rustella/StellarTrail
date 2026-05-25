//! Removes public knot difficulty metadata from the catalog schema.
//!
//! Knots are now presented by usage category and type only. The old difficulty
//! column carried no imported data and should not remain part of the public
//! contract.

use sea_orm_migration::prelude::*;

/// Drops the nullable knot difficulty column.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Removes `knots.difficulty` from the final application schema.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE knots DROP COLUMN difficulty")
            .await?;
        Ok(())
    }

    /// Restores the column shape only; deleted difficulty values are not recoverable.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE knots ADD COLUMN difficulty TEXT NULL")
            .await?;
        Ok(())
    }
}
