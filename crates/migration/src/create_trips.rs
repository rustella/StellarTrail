//! Creates trip tables, members, invitations, preparation records, and outdoor experiences.

use sea_orm_migration::prelude::*;

/// Trip section record tables that share a JSON payload envelope for sparse editing.
const TRIP_RECORD_TABLES: &[&str] = &[
    "trip_personal_gear_items",
    "trip_shared_gear_demands",
    "trip_itinerary_days",
    "trip_itinerary_time_slots",
    "trip_route_segments",
    "trip_segment_assignments",
    "trip_food_meals",
    "trip_food_items",
    "trip_food_supplies",
    "trip_medical_items",
    "trip_safety_risks",
    "trip_rescue_contacts",
    "trip_budget_items",
    "trip_goals",
];

/// Migration that adds the first trip-center schema.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "create_trips"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates trip, member, invitation, typed record, and experience tables.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS trips (
                id TEXT PRIMARY KEY,
                owner_user_id TEXT NOT NULL REFERENCES users(id),
                trip_type TEXT NOT NULL DEFAULT 'team',
                title TEXT NOT NULL,
                description TEXT NULL,
                start_date TEXT NULL,
                end_date TEXT NULL,
                enabled_sections_json TEXT NOT NULL,
                route_use_slope_adjustment BOOLEAN NOT NULL DEFAULT FALSE,
                route_use_high_altitude_adjustment BOOLEAN NOT NULL DEFAULT FALSE,
                route_start_altitude_m INTEGER NULL,
                field_versions_json TEXT NOT NULL,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS trip_members (
                id TEXT PRIMARY KEY,
                trip_id TEXT NOT NULL REFERENCES trips(id),
                user_id TEXT NOT NULL REFERENCES users(id),
                profile_json TEXT NOT NULL,
                field_versions_json TEXT NOT NULL,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE (trip_id, user_id)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS trip_invitations (
                id TEXT PRIMARY KEY,
                trip_id TEXT NOT NULL REFERENCES trips(id),
                token TEXT NOT NULL UNIQUE,
                created_by_user_id TEXT NOT NULL REFERENCES users(id),
                revoked_at TEXT NULL,
                created_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS trip_sections (
                trip_id TEXT NOT NULL REFERENCES trips(id),
                section_key TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                enabled BOOLEAN NOT NULL DEFAULT TRUE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (trip_id, section_key)
            )"#,
        )
        .await?;
        for table in TRIP_RECORD_TABLES {
            db.execute_unprepared(&format!(
                r#"CREATE TABLE IF NOT EXISTS {table} (
                    id TEXT PRIMARY KEY,
                    trip_id TEXT NOT NULL REFERENCES trips(id),
                    parent_id TEXT NULL,
                    sort_order INTEGER NOT NULL DEFAULT 0,
                    payload_json TEXT NOT NULL,
                    field_versions_json TEXT NOT NULL,
                    created_by_user_id TEXT NULL REFERENCES users(id),
                    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )"#
            ))
            .await?;
            db.execute_unprepared(&format!(
                "CREATE INDEX IF NOT EXISTS idx_{table}_trip_active_order ON {table}(trip_id, is_deleted, sort_order)"
            ))
            .await?;
            db.execute_unprepared(&format!(
                "CREATE INDEX IF NOT EXISTS idx_{table}_parent_active ON {table}(parent_id, is_deleted)"
            ))
            .await?;
            db.execute_unprepared(&format!(
                "CREATE INDEX IF NOT EXISTS idx_{table}_created_by ON {table}(created_by_user_id)"
            ))
            .await?;
        }
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS outdoor_experiences (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id),
                source_trip_id TEXT NULL REFERENCES trips(id),
                trip_type TEXT NOT NULL,
                title TEXT NOT NULL,
                start_date TEXT NULL,
                end_date TEXT NULL,
                day_count INTEGER NOT NULL DEFAULT 0,
                companion_count INTEGER NOT NULL DEFAULT 0,
                route_summary TEXT NULL,
                gear_summary TEXT NULL,
                food_summary TEXT NULL,
                budget_summary TEXT NULL,
                notes TEXT NULL,
                snapshot_json TEXT NOT NULL,
                field_versions_json TEXT NOT NULL,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE (user_id, source_trip_id)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS trails (
                id TEXT PRIMARY KEY,
                owner_user_id TEXT NOT NULL REFERENCES users(id),
                display_name TEXT NOT NULL,
                description TEXT NULL,
                source_format TEXT NOT NULL,
                original_filename TEXT NOT NULL,
                content_type TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                sha256_hex TEXT NOT NULL,
                bucket TEXT NOT NULL,
                object_key TEXT NOT NULL,
                normalized_points_json TEXT NOT NULL,
                simplified_geojson_json TEXT NOT NULL,
                bounds_json TEXT NULL,
                distance_m REAL NOT NULL DEFAULT 0,
                ascent_m REAL NOT NULL DEFAULT 0,
                descent_m REAL NOT NULL DEFAULT 0,
                min_elevation_m REAL NULL,
                max_elevation_m REAL NULL,
                start_time TEXT NULL,
                end_time TEXT NULL,
                point_count INTEGER NOT NULL DEFAULT 0,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS trip_trails (
                trip_id TEXT NOT NULL REFERENCES trips(id),
                trail_id TEXT NOT NULL REFERENCES trails(id),
                linked_by_user_id TEXT NOT NULL REFERENCES users(id),
                role TEXT NOT NULL DEFAULT 'route',
                sort_order INTEGER NOT NULL DEFAULT 0,
                notes TEXT NULL,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (trip_id, trail_id)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS outdoor_experience_trails (
                outdoor_experience_id TEXT NOT NULL REFERENCES outdoor_experiences(id),
                trail_id TEXT NOT NULL REFERENCES trails(id),
                linked_by_user_id TEXT NOT NULL REFERENCES users(id),
                role TEXT NOT NULL DEFAULT 'route',
                sort_order INTEGER NOT NULL DEFAULT 0,
                notes TEXT NULL,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (outdoor_experience_id, trail_id)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS map_annotations (
                id TEXT PRIMARY KEY,
                owner_user_id TEXT NOT NULL REFERENCES users(id),
                trail_id TEXT NULL REFERENCES trails(id),
                lng REAL NOT NULL,
                lat REAL NOT NULL,
                elevation_m REAL NULL,
                trail_point_index INTEGER NULL,
                annotation_type TEXT NOT NULL,
                title TEXT NULL,
                note TEXT NULL,
                field_versions_json TEXT NOT NULL,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS trip_map_annotations (
                trip_id TEXT NOT NULL REFERENCES trips(id),
                annotation_id TEXT NOT NULL REFERENCES map_annotations(id),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                PRIMARY KEY (trip_id, annotation_id)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS outdoor_experience_map_annotations (
                outdoor_experience_id TEXT NOT NULL REFERENCES outdoor_experiences(id),
                annotation_id TEXT NOT NULL REFERENCES map_annotations(id),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                PRIMARY KEY (outdoor_experience_id, annotation_id)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_trips_owner_active_start ON trips(owner_user_id, is_deleted, start_date, updated_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_trip_members_user_active ON trip_members(user_id, is_deleted, trip_id)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_trip_members_trip_active ON trip_members(trip_id, is_deleted)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_outdoor_experiences_user_active ON outdoor_experiences(user_id, is_deleted, start_date, updated_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_trails_owner_active_updated ON trails(owner_user_id, is_deleted, updated_at)",
        )
        .await?;
        db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_trails_sha256 ON trails(sha256_hex)")
            .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_trip_trails_trip_active_order ON trip_trails(trip_id, is_deleted, sort_order)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_trip_trails_trail_active ON trip_trails(trail_id, is_deleted)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_outdoor_experience_trails_experience_active_order ON outdoor_experience_trails(outdoor_experience_id, is_deleted, sort_order)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_outdoor_experience_trails_trail_active ON outdoor_experience_trails(trail_id, is_deleted)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_map_annotations_owner_active ON map_annotations(owner_user_id, is_deleted, updated_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_map_annotations_trail_active ON map_annotations(trail_id, is_deleted)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_trip_map_annotations_trip_active ON trip_map_annotations(trip_id, is_deleted)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_outdoor_experience_map_annotations_experience_active ON outdoor_experience_map_annotations(outdoor_experience_id, is_deleted)",
        )
        .await?;
        Ok(())
    }

    /// Drops trip-center tables and indexes.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "DROP INDEX IF EXISTS idx_outdoor_experience_map_annotations_experience_active",
        )
        .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_trip_map_annotations_trip_active")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_map_annotations_trail_active")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_map_annotations_owner_active")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_outdoor_experience_trails_trail_active")
            .await?;
        db.execute_unprepared(
            "DROP INDEX IF EXISTS idx_outdoor_experience_trails_experience_active_order",
        )
        .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_trip_trails_trail_active")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_trip_trails_trip_active_order")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_trails_sha256")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_trails_owner_active_updated")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_outdoor_experiences_user_active")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_trip_members_trip_active")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_trip_members_user_active")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_trips_owner_active_start")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS outdoor_experience_map_annotations")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS trip_map_annotations")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS map_annotations")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS outdoor_experience_trails")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS trip_trails")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS trails").await?;
        db.execute_unprepared("DROP TABLE IF EXISTS outdoor_experiences")
            .await?;
        for table in TRIP_RECORD_TABLES.iter().rev() {
            db.execute_unprepared(&format!("DROP INDEX IF EXISTS idx_{table}_created_by"))
                .await?;
            db.execute_unprepared(&format!("DROP INDEX IF EXISTS idx_{table}_parent_active"))
                .await?;
            db.execute_unprepared(&format!(
                "DROP INDEX IF EXISTS idx_{table}_trip_active_order"
            ))
            .await?;
            db.execute_unprepared(&format!("DROP TABLE IF EXISTS {table}"))
                .await?;
        }
        db.execute_unprepared("DROP TABLE IF EXISTS trip_sections")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS trip_invitations")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS trip_members")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS trips").await?;
        Ok(())
    }
}
