//! Compatibility registrations for schema patches folded into base migrations.
//!
//! These migrations used to exist as standalone schema patches. The current
//! base migrations already create the folded columns and indexes on fresh
//! databases, but existing databases can still have these version names in
//! `seaql_migrations`. Keeping no-op registrations lets SeaORM validate that
//! history without replaying obsolete DDL.

use sea_orm_migration::prelude::*;

macro_rules! folded_migration {
    ($module:ident, $name:literal) => {
        pub mod $module {
            use super::*;

            /// No-op marker for a migration already folded into base schema.
            pub struct Migration;

            impl MigrationName for Migration {
                fn name(&self) -> &str {
                    $name
                }
            }

            #[async_trait::async_trait]
            impl MigrationTrait for Migration {
                async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
                    Ok(())
                }

                async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
                    Ok(())
                }
            }
        }
    };
}

folded_migration!(add_password_auth, "m20260516_000003_add_password_auth");
folded_migration!(add_refresh_tokens, "m20260516_000008_add_refresh_tokens");
folded_migration!(
    add_email_code_failed_attempts,
    "m20260518_000001_add_email_code_failed_attempts"
);
folded_migration!(
    add_gear_specs_prices,
    "m20260518_000004_add_gear_specs_prices"
);
folded_migration!(
    add_gear_atlas_import_metadata,
    "m20260521_000002_add_gear_atlas_import_metadata"
);
folded_migration!(add_gear_variants, "m20260522_000001_add_gear_variants");
folded_migration!(
    add_gear_atlas_review_snapshots,
    "m20260523_000001_add_gear_atlas_review_snapshots"
);
folded_migration!(
    remove_knot_difficulty,
    "m20260523_000002_remove_knot_difficulty"
);
folded_migration!(
    add_upload_images_user_purpose_index,
    "m20260523_000003_add_upload_images_user_purpose_index"
);
folded_migration!(
    add_soft_delete_flags,
    "m20260523_000004_add_soft_delete_flags"
);
folded_migration!(add_gear_quantities, "m20260524_000004_add_gear_quantities");
folded_migration!(
    add_knot_localization_aliases,
    "m20260525_000001_add_knot_localization_aliases"
);
folded_migration!(create_team_trip_plans, "create_team_trip_plans");
folded_migration!(
    ensure_gear_atlas_import_i18n,
    "m20260611_000001_ensure_gear_atlas_import_i18n"
);
