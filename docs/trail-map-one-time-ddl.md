# Trail Map One-Time DDL

The fresh schema folds trail and map tables into `crates/migration/src/create_trips.rs`. Existing PostgreSQL databases that have already passed the original `create_trips` migration need this one-time DDL during rollout.

Run this in a maintenance window after deploying code that understands the new tables. The operation is additive and does not backfill user data because there were no persisted trail assets before this feature.

```sql
BEGIN;

CREATE TABLE IF NOT EXISTS trails (
    id TEXT PRIMARY KEY,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    display_name TEXT NOT NULL,
    description TEXT NULL,
    source_format TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    sha256_hex TEXT NOT NULL,
    bucket TEXT NOT NULL,
    object_key TEXT NOT NULL,
    normalized_points_json TEXT NOT NULL,
    simplified_geojson_json TEXT NOT NULL,
    bounds_json TEXT NULL,
    distance_m DOUBLE PRECISION NOT NULL DEFAULT 0,
    ascent_m DOUBLE PRECISION NOT NULL DEFAULT 0,
    descent_m DOUBLE PRECISION NOT NULL DEFAULT 0,
    min_elevation_m DOUBLE PRECISION NULL,
    max_elevation_m DOUBLE PRECISION NULL,
    start_time TEXT NULL,
    end_time TEXT NULL,
    point_count BIGINT NOT NULL DEFAULT 0,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS trip_trails (
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
);

CREATE TABLE IF NOT EXISTS outdoor_experience_trails (
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
);

CREATE TABLE IF NOT EXISTS map_annotations (
    id TEXT PRIMARY KEY,
    owner_user_id TEXT NOT NULL REFERENCES users(id),
    trail_id TEXT NULL REFERENCES trails(id),
    lng DOUBLE PRECISION NOT NULL,
    lat DOUBLE PRECISION NOT NULL,
    elevation_m DOUBLE PRECISION NULL,
    trail_point_index BIGINT NULL,
    annotation_type TEXT NOT NULL,
    title TEXT NULL,
    note TEXT NULL,
    field_versions_json TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS trip_map_annotations (
    trip_id TEXT NOT NULL REFERENCES trips(id),
    annotation_id TEXT NOT NULL REFERENCES map_annotations(id),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (trip_id, annotation_id)
);

CREATE TABLE IF NOT EXISTS outdoor_experience_map_annotations (
    outdoor_experience_id TEXT NOT NULL REFERENCES outdoor_experiences(id),
    annotation_id TEXT NOT NULL REFERENCES map_annotations(id),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (outdoor_experience_id, annotation_id)
);

CREATE INDEX IF NOT EXISTS idx_trails_owner_active_updated
    ON trails(owner_user_id, is_deleted, updated_at);
CREATE INDEX IF NOT EXISTS idx_trails_sha256 ON trails(sha256_hex);
CREATE INDEX IF NOT EXISTS idx_trip_trails_trip_active_order
    ON trip_trails(trip_id, is_deleted, sort_order);
CREATE INDEX IF NOT EXISTS idx_trip_trails_trail_active
    ON trip_trails(trail_id, is_deleted);
CREATE INDEX IF NOT EXISTS idx_outdoor_experience_trails_experience_active_order
    ON outdoor_experience_trails(outdoor_experience_id, is_deleted, sort_order);
CREATE INDEX IF NOT EXISTS idx_outdoor_experience_trails_trail_active
    ON outdoor_experience_trails(trail_id, is_deleted);
CREATE INDEX IF NOT EXISTS idx_map_annotations_owner_active
    ON map_annotations(owner_user_id, is_deleted, updated_at);
CREATE INDEX IF NOT EXISTS idx_map_annotations_trail_active
    ON map_annotations(trail_id, is_deleted);
CREATE INDEX IF NOT EXISTS idx_trip_map_annotations_trip_active
    ON trip_map_annotations(trip_id, is_deleted);
CREATE INDEX IF NOT EXISTS idx_outdoor_experience_map_annotations_experience_active
    ON outdoor_experience_map_annotations(outdoor_experience_id, is_deleted);

COMMIT;
```

Backfill checklist:

- No historical trail rows are inserted. Existing route segments remain separate from the new trail library.
- No historical trip or outdoor experience annotations are inserted.
- Trip-to-outdoor-experience conversion copies only trails owned by the converting user after this DDL is present.
- Confirm object storage has a private bucket configured for uploaded trail source files before enabling upload endpoints.
