use sea_orm::{ConnectionTrait, Database, QueryResult, Statement};
use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_migration::Migrator;

#[tokio::test]
async fn fresh_schema_contains_folded_migration_columns() {
    let db = Database::connect("sqlite::memory:").await.expect("connect");
    Migrator::up(&db, None).await.expect("migrate");

    for (table, columns) in [
        (
            "users",
            &[
                "username",
                "email",
                "password_hash",
                "phone",
                "phone_bound_at",
                "failed_login_attempts",
                "last_failed_login_at",
            ][..],
        ),
        (
            "sms_verification_challenges",
            &["phone", "purpose", "out_id", "expires_at", "consumed_at"][..],
        ),
        (
            "sessions",
            &["refresh_token_hash", "refresh_expires_at", "refreshed_at"][..],
        ),
        ("email_verification_codes", &["failed_attempts"][..]),
        (
            "user_gear_items",
            &[
                "official_price_cents",
                "official_price_currency",
                "purchase_price_currency",
                "specs_json",
                "atlas_item_id",
                "selected_variant_key",
                "selected_variant_label",
                "quantity",
                "archived_at",
                "is_deleted",
            ][..],
        ),
        (
            "gear_atlas_items",
            &[
                "variants_json",
                "submitted_snapshot_json",
                "review_changes_json",
                "source_key",
                "source_name",
                "source_url",
                "source_license_note",
                "import_batch_id",
                "imported_at",
                "source_rating_score",
                "source_rating_count",
                "is_deleted",
            ][..],
        ),
        (
            "gear_atlas_item_localizations",
            &[
                "variants_json",
                "specs_json",
                "translation_status",
                "translation_provider",
                "translated_at",
            ][..],
        ),
        (
            "gear_atlas_import_sources",
            &[
                "source_key",
                "canonical_key",
                "atlas_item_id",
                "source_locale",
                "detail_score",
                "last_seen_batch_id",
                "last_action",
            ][..],
        ),
        ("upload_images", &["is_deleted"][..]),
        ("user_feedback", &["is_deleted"][..]),
        (
            "gear_packing_list_items",
            &["planned_quantity", "packed_quantity"][..],
        ),
        ("knot_localizations", &["aliases_json"][..]),
        (
            "user_outdoor_profiles",
            &[
                "outdoor_id",
                "real_name",
                "gender",
                "birth_date",
                "height_cm",
                "phone",
                "emergency_contact",
                "emergency_contact_relationship",
                "emergency_phone",
                "blood_type",
                "medical_history",
                "allergy_history",
                "medical_response_note",
                "diet_preference",
                "insurance_policy_no",
                "insurance_company_phone",
                "experience_note",
            ][..],
        ),
        ("trip_shared_gear_demands", &["created_by_user_id"][..]),
        (
            "app_content_pages",
            &["page_key", "client_key", "locale", "content_json", "status"][..],
        ),
        (
            "trips",
            &[
                "route_use_slope_adjustment",
                "route_use_high_altitude_adjustment",
                "route_start_altitude_m",
            ][..],
        ),
    ] {
        for column in columns {
            assert!(
                table_has_column(&db, table, column).await,
                "missing {table}.{column}"
            );
        }
    }

    for legacy_column in [
        "color",
        "material",
        "capacity",
        "size",
        "warmth_index",
        "waterproof_index",
        "expiry_or_warranty_date",
    ] {
        assert!(
            !table_has_column(&db, "user_gear_items", legacy_column).await,
            "legacy personal gear column still exists: {legacy_column}"
        );
    }

    assert!(
        !table_has_column(&db, "knots", "difficulty").await,
        "knots.difficulty should be folded out of the base schema"
    );
    assert_eq!(
        column_type(&db, "media_resources", "size_bytes").await,
        Some("BIGINT".to_owned())
    );
}

#[test]
fn folded_schema_patch_migrations_keep_history_compatibility() {
    let names = Migrator::migrations()
        .into_iter()
        .map(|migration| migration.name().to_owned())
        .collect::<Vec<_>>();

    for compatibility_name in [
        "m20260516_000003_add_password_auth",
        "m20260516_000008_add_refresh_tokens",
        "m20260518_000001_add_email_code_failed_attempts",
        "m20260518_000004_add_gear_specs_prices",
        "m20260521_000002_add_gear_atlas_import_metadata",
        "m20260522_000001_add_gear_variants",
        "m20260523_000001_add_gear_atlas_review_snapshots",
        "m20260523_000002_remove_knot_difficulty",
        "m20260523_000003_add_upload_images_user_purpose_index",
        "m20260523_000004_add_soft_delete_flags",
        "m20260524_000004_add_gear_quantities",
        "m20260525_000001_add_knot_localization_aliases",
        "create_team_trip_plans",
    ] {
        assert!(
            names.iter().any(|name| name == compatibility_name),
            "folded migration history marker is missing: {compatibility_name}"
        );
    }

    let unregistered_name = "m20260526_000001_alter_media_resources_size_bytes_bigint";
    assert!(
        !names.iter().any(|name| name == unregistered_name),
        "folded migration is unexpectedly registered: {unregistered_name}"
    );
}

async fn table_has_column(db: &sea_orm::DatabaseConnection, table: &str, column: &str) -> bool {
    table_columns(db, table).await.into_iter().any(|row| {
        let name: String = row.try_get("", "name").expect("column name");
        name == column
    })
}

async fn column_type(
    db: &sea_orm::DatabaseConnection,
    table: &str,
    column: &str,
) -> Option<String> {
    table_columns(db, table).await.into_iter().find_map(|row| {
        let name: String = row.try_get("", "name").expect("column name");
        if name == column {
            Some(row.try_get("", "type").expect("column type"))
        } else {
            None
        }
    })
}

async fn table_columns(db: &sea_orm::DatabaseConnection, table: &str) -> Vec<QueryResult> {
    db.query_all(Statement::from_string(
        db.get_database_backend(),
        format!("PRAGMA table_info({table})"),
    ))
    .await
    .expect("table info")
}
