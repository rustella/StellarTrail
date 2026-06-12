//! Repository for reusable trail assets, context links, and map annotations.

use std::collections::{BTreeMap, BTreeSet};

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, QueryResult, Value};
use serde_json::Value as JsonValue;
use stellartrail_domain::{
    gear::now_rfc3339,
    trail::{
        MapAnnotation, MapAnnotationDraft, MapAnnotationPatch, MapTrailLink, Trail, TrailBounds,
        TrailCreateDraft, TrailMetadataPatch, TrailPoint, TrailSourceFormat, TrailSummary,
        TripOverviewTrail,
    },
    trip::{FieldConflict, FieldVersions},
    validation::ValidationError,
};
use uuid::Uuid;

use super::statement;

/// Error boundary for trail persistence and permission checks.
#[derive(Debug)]
pub enum TrailRepositoryError {
    Db(DbErr),
    Validation(ValidationError),
    Conflict(Vec<FieldConflict>),
    Forbidden,
}

impl From<DbErr> for TrailRepositoryError {
    fn from(value: DbErr) -> Self {
        Self::Db(value)
    }
}

impl From<ValidationError> for TrailRepositoryError {
    fn from(value: ValidationError) -> Self {
        Self::Validation(value)
    }
}

/// Persistence boundary for trails, context links, and map annotations.
#[derive(Clone)]
pub struct TrailRepository {
    db: DatabaseConnection,
}

impl TrailRepository {
    /// Creates a repository bound to a database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Lists active trails owned by the current user.
    pub async fn list_owned(&self, user_id: &str) -> Result<Vec<TrailSummary>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                trail_select_sql(
                    "FROM trails t \
                     WHERE t.owner_user_id = ? AND t.is_deleted = FALSE \
                     ORDER BY t.updated_at DESC, t.id DESC",
                ),
                vec![user_id.to_owned().into()],
            ))
            .await?;
        rows.iter()
            .map(map_trail)
            .map(|result| result.map(|trail| TrailSummary::from(&trail)))
            .collect()
    }

    /// Reads one active trail owned by the current user.
    pub async fn get_owned(&self, user_id: &str, trail_id: &str) -> Result<Option<Trail>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                trail_select_sql(
                    "FROM trails t \
                     WHERE t.id = ? AND t.owner_user_id = ? AND t.is_deleted = FALSE",
                ),
                vec![trail_id.to_owned().into(), user_id.to_owned().into()],
            ))
            .await?;
        row.as_ref().map(map_trail).transpose()
    }

    /// Reads one active owned trail by content hash so repeated imports can reuse it.
    pub async fn get_owned_by_sha256(
        &self,
        user_id: &str,
        sha256_hex: &str,
    ) -> Result<Option<Trail>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                trail_select_sql(
                    "FROM trails t \
                     WHERE t.owner_user_id = ? AND t.sha256_hex = ? AND t.is_deleted = FALSE \
                     ORDER BY t.updated_at DESC, t.id DESC \
                     LIMIT 1",
                ),
                vec![user_id.to_owned().into(), sha256_hex.to_owned().into()],
            ))
            .await?;
        row.as_ref().map(map_trail).transpose()
    }

    /// Reads one active trail when the user owns it or can see it through a linked context.
    pub async fn get_visible(&self, user_id: &str, trail_id: &str) -> Result<Option<Trail>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                trail_select_sql(
                    "FROM trails t \
                     WHERE t.id = ? AND t.is_deleted = FALSE AND ( \
                        t.owner_user_id = ? \
                        OR EXISTS ( \
                            SELECT 1 FROM trip_trails tt \
                            JOIN trip_members tm ON tm.trip_id = tt.trip_id \
                                AND tm.user_id = ? AND tm.is_deleted = FALSE \
                            JOIN trips trip ON trip.id = tt.trip_id AND trip.is_deleted = FALSE \
                            WHERE tt.trail_id = t.id AND tt.is_deleted = FALSE \
                        ) \
                        OR EXISTS ( \
                            SELECT 1 FROM outdoor_experience_trails ot \
                            JOIN outdoor_experiences oe ON oe.id = ot.outdoor_experience_id \
                                AND oe.user_id = ? AND oe.is_deleted = FALSE \
                            WHERE ot.trail_id = t.id AND ot.is_deleted = FALSE \
                        ) \
                     )",
                ),
                vec![
                    trail_id.to_owned().into(),
                    user_id.to_owned().into(),
                    user_id.to_owned().into(),
                    user_id.to_owned().into(),
                ],
            ))
            .await?;
        row.as_ref().map(map_trail).transpose()
    }

    /// Creates an active reusable trail owned by the current user.
    pub async fn create(
        &self,
        user_id: &str,
        draft: &TrailCreateDraft,
    ) -> Result<Trail, TrailRepositoryError> {
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO trails \
                 (id, owner_user_id, display_name, description, source_format, original_filename, \
                  content_type, size_bytes, sha256_hex, bucket, object_key, normalized_points_json, \
                  simplified_geojson_json, bounds_json, distance_m, ascent_m, descent_m, \
                  min_elevation_m, max_elevation_m, start_time, end_time, point_count, \
                  is_deleted, created_at, updated_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, FALSE, ?, ?)",
                vec![
                    id.clone().into(),
                    user_id.to_owned().into(),
                    draft.display_name.clone().into(),
                    draft.description.clone().into(),
                    draft.source_format.as_str().to_owned().into(),
                    draft.original_filename.clone().into(),
                    draft.content_type.clone().into(),
                    draft.size_bytes.into(),
                    draft.sha256_hex.clone().into(),
                    draft.bucket.clone().into(),
                    draft.object_key.clone().into(),
                    json_string(&draft.normalized_points)?.into(),
                    json_string(&draft.simplified_geojson)?.into(),
                    optional_json_string(&draft.bounds)?.into(),
                    draft.distance_m.into(),
                    draft.ascent_m.into(),
                    draft.descent_m.into(),
                    draft.min_elevation_m.into(),
                    draft.max_elevation_m.into(),
                    draft.start_time.clone().into(),
                    draft.end_time.clone().into(),
                    draft.point_count.into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        self.get_owned(user_id, &id)
            .await?
            .ok_or_else(|| DbErr::Custom("created trail not found".to_owned()).into())
    }

    /// Updates owner-controlled trail metadata.
    pub async fn update_owned_metadata(
        &self,
        user_id: &str,
        trail_id: &str,
        patch: &TrailMetadataPatch,
    ) -> Result<Option<Trail>, TrailRepositoryError> {
        if self.get_owned(user_id, trail_id).await?.is_none() {
            return Ok(None);
        }
        let now = now_rfc3339();
        match (&patch.display_name, &patch.description) {
            (Some(display_name), Some(description)) => {
                self.db
                    .execute(statement(
                        self.db.get_database_backend(),
                        "UPDATE trails SET display_name = ?, description = ?, updated_at = ? \
                         WHERE id = ? AND owner_user_id = ? AND is_deleted = FALSE",
                        vec![
                            display_name.clone().into(),
                            description.clone().into(),
                            now.into(),
                            trail_id.to_owned().into(),
                            user_id.to_owned().into(),
                        ],
                    ))
                    .await?;
            }
            (Some(display_name), None) => {
                self.db
                    .execute(statement(
                        self.db.get_database_backend(),
                        "UPDATE trails SET display_name = ?, updated_at = ? \
                         WHERE id = ? AND owner_user_id = ? AND is_deleted = FALSE",
                        vec![
                            display_name.clone().into(),
                            now.into(),
                            trail_id.to_owned().into(),
                            user_id.to_owned().into(),
                        ],
                    ))
                    .await?;
            }
            (None, Some(description)) => {
                self.db
                    .execute(statement(
                        self.db.get_database_backend(),
                        "UPDATE trails SET description = ?, updated_at = ? \
                         WHERE id = ? AND owner_user_id = ? AND is_deleted = FALSE",
                        vec![
                            description.clone().into(),
                            now.into(),
                            trail_id.to_owned().into(),
                            user_id.to_owned().into(),
                        ],
                    ))
                    .await?;
            }
            (None, None) => {}
        }
        self.get_owned(user_id, trail_id).await.map_err(Into::into)
    }

    /// Soft-deletes an owner-controlled trail and hides it from every linked context.
    pub async fn delete_owned(
        &self,
        user_id: &str,
        trail_id: &str,
    ) -> Result<bool, TrailRepositoryError> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE trails SET is_deleted = TRUE, updated_at = ? \
                 WHERE id = ? AND owner_user_id = ? AND is_deleted = FALSE",
                vec![
                    now.into(),
                    trail_id.to_owned().into(),
                    user_id.to_owned().into(),
                ],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Returns the full map state for a trip member.
    pub async fn trip_map_state(
        &self,
        user_id: &str,
        trip_id: &str,
    ) -> Result<Option<(Vec<MapTrailLink>, Vec<MapAnnotation>)>, TrailRepositoryError> {
        if self
            .trip_owner_for_member(user_id, trip_id)
            .await?
            .is_none()
        {
            return Ok(None);
        }
        let trails = self.trip_trail_links(trip_id).await?;
        let annotations = self.trip_annotations(trip_id).await?;
        Ok(Some((trails, annotations)))
    }

    /// Returns map-ready trails linked to the current user's visible active trips.
    pub async fn trips_map_overview(
        &self,
        user_id: &str,
        max_trips: u64,
        max_trails: u64,
    ) -> Result<Vec<TripOverviewTrail>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                trail_select_sql(
                    ", vt.id AS overview_trip_id, vt.title AS overview_trip_title, \
                        vt.start_date AS overview_trip_start_date, \
                        vt.end_date AS overview_trip_end_date, \
                        l.linked_by_user_id AS link_linked_by_user_id, l.role AS link_role, \
                        l.sort_order AS link_sort_order, l.notes AS link_notes, \
                        l.created_at AS link_created_at, l.updated_at AS link_updated_at \
                     FROM ( \
                        SELECT p.id, p.title, p.start_date, p.end_date, p.updated_at \
                        FROM trips p \
                        JOIN trip_members m ON m.trip_id = p.id AND m.user_id = ? AND m.is_deleted = FALSE \
                        WHERE p.is_deleted = FALSE \
                        ORDER BY COALESCE(p.start_date, p.updated_at) DESC, p.updated_at DESC, p.id DESC \
                        LIMIT ? \
                     ) vt \
                     JOIN trip_trails l ON l.trip_id = vt.id AND l.is_deleted = FALSE \
                     JOIN trails t ON t.id = l.trail_id AND t.is_deleted = FALSE \
                     ORDER BY COALESCE(vt.start_date, vt.updated_at) DESC, vt.updated_at DESC, vt.id DESC, \
                        l.sort_order ASC, l.created_at ASC, t.id ASC \
                     LIMIT ?",
                ),
                vec![user_id.to_owned().into(), (max_trips as i64).into(), (max_trails as i64).into()],
            ))
            .await?;
        rows.iter().map(map_trip_overview_trail).collect()
    }

    /// Upload flow helper that links a newly created owned trail to a trip.
    pub async fn link_trail_to_trip(
        &self,
        user_id: &str,
        trip_id: &str,
        trail_id: &str,
        max_trails_per_trip: u64,
    ) -> Result<Option<MapTrailLink>, TrailRepositoryError> {
        if self
            .trip_owner_for_member(user_id, trip_id)
            .await?
            .is_none()
        {
            return Ok(None);
        }
        if self.get_owned(user_id, trail_id).await?.is_none() {
            return Ok(None);
        }
        if let Some(existing) = self.trip_trail_link(trip_id, trail_id).await? {
            return Ok(Some(existing));
        }
        self.enforce_trip_trail_limit(trip_id, max_trails_per_trip)
            .await?;
        self.upsert_trip_trail_link(user_id, trip_id, trail_id)
            .await?;
        self.trip_trail_link(trip_id, trail_id)
            .await
            .map_err(Into::into)
    }

    /// Soft-deletes a trip trail link when the actor owns either the trip or the trail.
    pub async fn unlink_trail_from_trip(
        &self,
        user_id: &str,
        trip_id: &str,
        trail_id: &str,
    ) -> Result<bool, TrailRepositoryError> {
        let Some((trip_owner_id, trail_owner_id)) =
            self.trip_link_owners(trip_id, trail_id).await?
        else {
            return Ok(false);
        };
        if user_id != trip_owner_id && user_id != trail_owner_id {
            return Err(TrailRepositoryError::Forbidden);
        }
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE trip_trails SET is_deleted = TRUE, updated_at = ? \
                 WHERE trip_id = ? AND trail_id = ? AND is_deleted = FALSE",
                vec![
                    now.into(),
                    trip_id.to_owned().into(),
                    trail_id.to_owned().into(),
                ],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Returns the full map state for an outdoor experience owner.
    pub async fn outdoor_experience_map_state(
        &self,
        user_id: &str,
        experience_id: &str,
    ) -> Result<Option<(Vec<MapTrailLink>, Vec<MapAnnotation>)>, TrailRepositoryError> {
        if !self
            .outdoor_experience_owned(user_id, experience_id)
            .await?
        {
            return Ok(None);
        }
        let trails = self.outdoor_experience_trail_links(experience_id).await?;
        let annotations = self.outdoor_experience_annotations(experience_id).await?;
        Ok(Some((trails, annotations)))
    }

    /// Links an owned trail to an owned outdoor experience.
    pub async fn link_trail_to_outdoor_experience(
        &self,
        user_id: &str,
        experience_id: &str,
        trail_id: &str,
    ) -> Result<Option<MapTrailLink>, TrailRepositoryError> {
        if !self
            .outdoor_experience_owned(user_id, experience_id)
            .await?
        {
            return Ok(None);
        }
        if self.get_owned(user_id, trail_id).await?.is_none() {
            return Ok(None);
        }
        if let Some(existing) = self
            .outdoor_experience_trail_link(experience_id, trail_id)
            .await?
        {
            return Ok(Some(existing));
        }
        self.upsert_outdoor_experience_trail_link(user_id, experience_id, trail_id)
            .await?;
        self.outdoor_experience_trail_link(experience_id, trail_id)
            .await
            .map_err(Into::into)
    }

    /// Soft-deletes a trail link from an owned outdoor experience.
    pub async fn unlink_trail_from_outdoor_experience(
        &self,
        user_id: &str,
        experience_id: &str,
        trail_id: &str,
    ) -> Result<bool, TrailRepositoryError> {
        if !self
            .outdoor_experience_owned(user_id, experience_id)
            .await?
        {
            return Ok(false);
        }
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE outdoor_experience_trails SET is_deleted = TRUE, updated_at = ? \
                 WHERE outdoor_experience_id = ? AND trail_id = ? AND is_deleted = FALSE",
                vec![
                    now.into(),
                    experience_id.to_owned().into(),
                    trail_id.to_owned().into(),
                ],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Creates an annotation scoped to one trip map.
    pub async fn create_trip_annotation(
        &self,
        user_id: &str,
        trip_id: &str,
        draft: &MapAnnotationDraft,
        max_annotations_per_context: u64,
    ) -> Result<Option<MapAnnotation>, TrailRepositoryError> {
        if self
            .trip_owner_for_member(user_id, trip_id)
            .await?
            .is_none()
        {
            return Ok(None);
        }
        if let Some(trail_id) = &draft.trail_id
            && self.trip_trail_link(trip_id, trail_id).await?.is_none()
        {
            return Err(ValidationError::single(
                "trail_id",
                "must reference a trail linked to this trip",
            )
            .into());
        }
        self.enforce_trip_annotation_limit(trip_id, max_annotations_per_context)
            .await?;
        let annotation = self.insert_annotation(user_id, draft).await?;
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO trip_map_annotations \
                 (trip_id, annotation_id, created_at, updated_at, is_deleted) \
                 VALUES (?, ?, ?, ?, FALSE)",
                vec![
                    trip_id.to_owned().into(),
                    annotation.id.clone().into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        Ok(Some(annotation))
    }

    /// Creates an annotation scoped to one outdoor experience map.
    pub async fn create_outdoor_experience_annotation(
        &self,
        user_id: &str,
        experience_id: &str,
        draft: &MapAnnotationDraft,
        max_annotations_per_context: u64,
    ) -> Result<Option<MapAnnotation>, TrailRepositoryError> {
        if !self
            .outdoor_experience_owned(user_id, experience_id)
            .await?
        {
            return Ok(None);
        }
        if let Some(trail_id) = &draft.trail_id
            && self
                .outdoor_experience_trail_link(experience_id, trail_id)
                .await?
                .is_none()
        {
            return Err(ValidationError::single(
                "trail_id",
                "must reference a trail linked to this outdoor experience",
            )
            .into());
        }
        self.enforce_outdoor_experience_annotation_limit(
            experience_id,
            max_annotations_per_context,
        )
        .await?;
        let annotation = self.insert_annotation(user_id, draft).await?;
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO outdoor_experience_map_annotations \
                 (outdoor_experience_id, annotation_id, created_at, updated_at, is_deleted) \
                 VALUES (?, ?, ?, ?, FALSE)",
                vec![
                    experience_id.to_owned().into(),
                    annotation.id.clone().into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        Ok(Some(annotation))
    }

    /// Updates one trip-scoped annotation when the actor owns the annotation or trip.
    pub async fn update_trip_annotation(
        &self,
        user_id: &str,
        trip_id: &str,
        annotation_id: &str,
        patch: &MapAnnotationPatch,
        base_versions: FieldVersions,
        force_fields: BTreeSet<String>,
    ) -> Result<Option<MapAnnotation>, TrailRepositoryError> {
        let Some(trip_owner_id) = self.trip_owner_for_member(user_id, trip_id).await? else {
            return Ok(None);
        };
        let Some(annotation) = self.trip_annotation(trip_id, annotation_id).await? else {
            return Ok(None);
        };
        if user_id != annotation.owner_user_id && user_id != trip_owner_id {
            return Err(TrailRepositoryError::Forbidden);
        }
        self.update_annotation(annotation, patch, base_versions, force_fields)
            .await
    }

    /// Updates one outdoor-experience-scoped annotation for the experience owner.
    pub async fn update_outdoor_experience_annotation(
        &self,
        user_id: &str,
        experience_id: &str,
        annotation_id: &str,
        patch: &MapAnnotationPatch,
        base_versions: FieldVersions,
        force_fields: BTreeSet<String>,
    ) -> Result<Option<MapAnnotation>, TrailRepositoryError> {
        if !self
            .outdoor_experience_owned(user_id, experience_id)
            .await?
        {
            return Ok(None);
        }
        let Some(annotation) = self
            .outdoor_experience_annotation(experience_id, annotation_id)
            .await?
        else {
            return Ok(None);
        };
        self.update_annotation(annotation, patch, base_versions, force_fields)
            .await
    }

    /// Deletes a trip-scoped annotation when the actor owns the annotation or trip.
    pub async fn delete_trip_annotation(
        &self,
        user_id: &str,
        trip_id: &str,
        annotation_id: &str,
    ) -> Result<bool, TrailRepositoryError> {
        let Some(trip_owner_id) = self.trip_owner_for_member(user_id, trip_id).await? else {
            return Ok(false);
        };
        let Some(annotation) = self.trip_annotation(trip_id, annotation_id).await? else {
            return Ok(false);
        };
        if user_id != annotation.owner_user_id && user_id != trip_owner_id {
            return Err(TrailRepositoryError::Forbidden);
        }
        let now = now_rfc3339();
        self.soft_delete_annotation_context(
            annotation_id,
            "trip_map_annotations",
            "trip_id",
            trip_id,
            &now,
        )
        .await
    }

    /// Deletes an outdoor-experience-scoped annotation for the experience owner.
    pub async fn delete_outdoor_experience_annotation(
        &self,
        user_id: &str,
        experience_id: &str,
        annotation_id: &str,
    ) -> Result<bool, TrailRepositoryError> {
        if !self
            .outdoor_experience_owned(user_id, experience_id)
            .await?
        {
            return Ok(false);
        }
        if self
            .outdoor_experience_annotation(experience_id, annotation_id)
            .await?
            .is_none()
        {
            return Ok(false);
        }
        let now = now_rfc3339();
        self.soft_delete_annotation_context(
            annotation_id,
            "outdoor_experience_map_annotations",
            "outdoor_experience_id",
            experience_id,
            &now,
        )
        .await
    }

    async fn trip_trail_links(&self, trip_id: &str) -> Result<Vec<MapTrailLink>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                trail_link_select_sql(
                    "trip_trails",
                    "l.trip_id = ? AND l.is_deleted = FALSE AND t.is_deleted = FALSE \
                     ORDER BY l.sort_order ASC, l.created_at ASC, t.id ASC",
                ),
                vec![trip_id.to_owned().into()],
            ))
            .await?;
        rows.iter().map(map_trail_link).collect()
    }

    async fn trip_trail_link(
        &self,
        trip_id: &str,
        trail_id: &str,
    ) -> Result<Option<MapTrailLink>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                trail_link_select_sql(
                    "trip_trails",
                    "l.trip_id = ? AND l.trail_id = ? AND l.is_deleted = FALSE AND t.is_deleted = FALSE",
                ),
                vec![trip_id.to_owned().into(), trail_id.to_owned().into()],
            ))
            .await?;
        row.as_ref().map(map_trail_link).transpose()
    }

    async fn outdoor_experience_trail_links(
        &self,
        experience_id: &str,
    ) -> Result<Vec<MapTrailLink>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                trail_link_select_sql(
                    "outdoor_experience_trails",
                    "l.outdoor_experience_id = ? AND l.is_deleted = FALSE AND t.is_deleted = FALSE \
                     ORDER BY l.sort_order ASC, l.created_at ASC, t.id ASC",
                ),
                vec![experience_id.to_owned().into()],
            ))
            .await?;
        rows.iter().map(map_trail_link).collect()
    }

    async fn outdoor_experience_trail_link(
        &self,
        experience_id: &str,
        trail_id: &str,
    ) -> Result<Option<MapTrailLink>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                trail_link_select_sql(
                    "outdoor_experience_trails",
                    "l.outdoor_experience_id = ? AND l.trail_id = ? \
                     AND l.is_deleted = FALSE AND t.is_deleted = FALSE",
                ),
                vec![experience_id.to_owned().into(), trail_id.to_owned().into()],
            ))
            .await?;
        row.as_ref().map(map_trail_link).transpose()
    }

    async fn upsert_trip_trail_link(
        &self,
        user_id: &str,
        trip_id: &str,
        trail_id: &str,
    ) -> Result<(), DbErr> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE trip_trails SET linked_by_user_id = ?, role = 'route', sort_order = 0, \
                 notes = NULL, is_deleted = FALSE, updated_at = ? \
                 WHERE trip_id = ? AND trail_id = ?",
                vec![
                    user_id.to_owned().into(),
                    now.clone().into(),
                    trip_id.to_owned().into(),
                    trail_id.to_owned().into(),
                ],
            ))
            .await?;
        if result.rows_affected() == 0 {
            self.db
                .execute(statement(
                    self.db.get_database_backend(),
                    "INSERT INTO trip_trails \
                     (trip_id, trail_id, linked_by_user_id, role, sort_order, notes, is_deleted, created_at, updated_at) \
                     VALUES (?, ?, ?, 'route', 0, NULL, FALSE, ?, ?)",
                    vec![
                        trip_id.to_owned().into(),
                        trail_id.to_owned().into(),
                        user_id.to_owned().into(),
                        now.clone().into(),
                        now.into(),
                    ],
                ))
                .await?;
        }
        Ok(())
    }

    async fn upsert_outdoor_experience_trail_link(
        &self,
        user_id: &str,
        experience_id: &str,
        trail_id: &str,
    ) -> Result<(), DbErr> {
        let now = now_rfc3339();
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE outdoor_experience_trails SET linked_by_user_id = ?, role = 'route', \
                 sort_order = 0, notes = NULL, is_deleted = FALSE, updated_at = ? \
                 WHERE outdoor_experience_id = ? AND trail_id = ?",
                vec![
                    user_id.to_owned().into(),
                    now.clone().into(),
                    experience_id.to_owned().into(),
                    trail_id.to_owned().into(),
                ],
            ))
            .await?;
        if result.rows_affected() == 0 {
            self.db
                .execute(statement(
                    self.db.get_database_backend(),
                    "INSERT INTO outdoor_experience_trails \
                     (outdoor_experience_id, trail_id, linked_by_user_id, role, sort_order, notes, \
                      is_deleted, created_at, updated_at) \
                     VALUES (?, ?, ?, 'route', 0, NULL, FALSE, ?, ?)",
                    vec![
                        experience_id.to_owned().into(),
                        trail_id.to_owned().into(),
                        user_id.to_owned().into(),
                        now.clone().into(),
                        now.into(),
                    ],
                ))
                .await?;
        }
        Ok(())
    }

    async fn trip_owner_for_member(
        &self,
        user_id: &str,
        trip_id: &str,
    ) -> Result<Option<String>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT p.owner_user_id \
                 FROM trips p \
                 JOIN trip_members m ON m.trip_id = p.id AND m.user_id = ? AND m.is_deleted = FALSE \
                 WHERE p.id = ? AND p.is_deleted = FALSE",
                vec![user_id.to_owned().into(), trip_id.to_owned().into()],
            ))
            .await?;
        row.map(|row| row.try_get("", "owner_user_id")).transpose()
    }

    async fn outdoor_experience_owned(
        &self,
        user_id: &str,
        experience_id: &str,
    ) -> Result<bool, DbErr> {
        Ok(self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT id FROM outdoor_experiences \
                 WHERE id = ? AND user_id = ? AND is_deleted = FALSE",
                vec![experience_id.to_owned().into(), user_id.to_owned().into()],
            ))
            .await?
            .is_some())
    }

    async fn trip_link_owners(
        &self,
        trip_id: &str,
        trail_id: &str,
    ) -> Result<Option<(String, String)>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT p.owner_user_id AS trip_owner_user_id, t.owner_user_id AS trail_owner_user_id \
                 FROM trip_trails l \
                 JOIN trips p ON p.id = l.trip_id AND p.is_deleted = FALSE \
                 JOIN trails t ON t.id = l.trail_id AND t.is_deleted = FALSE \
                 WHERE l.trip_id = ? AND l.trail_id = ? AND l.is_deleted = FALSE",
                vec![trip_id.to_owned().into(), trail_id.to_owned().into()],
            ))
            .await?;
        row.map(|row| {
            Ok((
                row.try_get("", "trip_owner_user_id")?,
                row.try_get("", "trail_owner_user_id")?,
            ))
        })
        .transpose()
    }

    async fn enforce_trip_trail_limit(
        &self,
        trip_id: &str,
        max_trails_per_trip: u64,
    ) -> Result<(), TrailRepositoryError> {
        let count = self.count_active("trip_trails", "trip_id", trip_id).await?;
        if u64::try_from(count.max(0)).unwrap_or(u64::MAX) >= max_trails_per_trip {
            return Err(ValidationError::single(
                "trail_id",
                format!("trip can link at most {max_trails_per_trip} trails"),
            )
            .into());
        }
        Ok(())
    }

    async fn enforce_trip_annotation_limit(
        &self,
        trip_id: &str,
        max_annotations_per_context: u64,
    ) -> Result<(), TrailRepositoryError> {
        let count = self
            .count_active("trip_map_annotations", "trip_id", trip_id)
            .await?;
        if u64::try_from(count.max(0)).unwrap_or(u64::MAX) >= max_annotations_per_context {
            return Err(ValidationError::single(
                "annotation",
                format!(
                    "map context can contain at most {max_annotations_per_context} annotations"
                ),
            )
            .into());
        }
        Ok(())
    }

    async fn enforce_outdoor_experience_annotation_limit(
        &self,
        experience_id: &str,
        max_annotations_per_context: u64,
    ) -> Result<(), TrailRepositoryError> {
        let count = self
            .count_active(
                "outdoor_experience_map_annotations",
                "outdoor_experience_id",
                experience_id,
            )
            .await?;
        if u64::try_from(count.max(0)).unwrap_or(u64::MAX) >= max_annotations_per_context {
            return Err(ValidationError::single(
                "annotation",
                format!(
                    "map context can contain at most {max_annotations_per_context} annotations"
                ),
            )
            .into());
        }
        Ok(())
    }

    async fn count_active(&self, table: &str, column: &str, id: &str) -> Result<i64, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!("SELECT COUNT(*) AS count FROM {table} WHERE {column} = ? AND is_deleted = FALSE"),
                vec![id.to_owned().into()],
            ))
            .await?;
        Ok(row
            .as_ref()
            .map(|row| row.try_get("", "count"))
            .transpose()?
            .unwrap_or(0))
    }

    async fn insert_annotation(
        &self,
        user_id: &str,
        draft: &MapAnnotationDraft,
    ) -> Result<MapAnnotation, TrailRepositoryError> {
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        let versions = initial_versions(["annotation_type", "title", "note", "elevation_m"]);
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO map_annotations \
                 (id, owner_user_id, trail_id, lng, lat, elevation_m, trail_point_index, \
                  annotation_type, title, note, field_versions_json, is_deleted, created_at, updated_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, FALSE, ?, ?)",
                vec![
                    id.clone().into(),
                    user_id.to_owned().into(),
                    draft.trail_id.clone().into(),
                    draft.lng.into(),
                    draft.lat.into(),
                    draft.elevation_m.into(),
                    draft.trail_point_index.into(),
                    draft.annotation_type.clone().into(),
                    draft.title.clone().into(),
                    draft.note.clone().into(),
                    json_string(&versions)?.into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        self.annotation(&id)
            .await?
            .ok_or_else(|| DbErr::Custom("created annotation not found".to_owned()).into())
    }

    async fn trip_annotations(&self, trip_id: &str) -> Result<Vec<MapAnnotation>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                annotation_select_sql(
                    "JOIN trip_map_annotations c ON c.annotation_id = a.id \
                     WHERE c.trip_id = ? AND c.is_deleted = FALSE AND a.is_deleted = FALSE \
                     ORDER BY a.created_at ASC, a.id ASC",
                ),
                vec![trip_id.to_owned().into()],
            ))
            .await?;
        rows.iter().map(map_annotation).collect()
    }

    async fn trip_annotation(
        &self,
        trip_id: &str,
        annotation_id: &str,
    ) -> Result<Option<MapAnnotation>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                annotation_select_sql(
                    "JOIN trip_map_annotations c ON c.annotation_id = a.id \
                     WHERE c.trip_id = ? AND a.id = ? AND c.is_deleted = FALSE AND a.is_deleted = FALSE",
                ),
                vec![trip_id.to_owned().into(), annotation_id.to_owned().into()],
            ))
            .await?;
        row.as_ref().map(map_annotation).transpose()
    }

    async fn outdoor_experience_annotations(
        &self,
        experience_id: &str,
    ) -> Result<Vec<MapAnnotation>, DbErr> {
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                annotation_select_sql(
                    "JOIN outdoor_experience_map_annotations c ON c.annotation_id = a.id \
                     WHERE c.outdoor_experience_id = ? AND c.is_deleted = FALSE AND a.is_deleted = FALSE \
                     ORDER BY a.created_at ASC, a.id ASC",
                ),
                vec![experience_id.to_owned().into()],
            ))
            .await?;
        rows.iter().map(map_annotation).collect()
    }

    async fn outdoor_experience_annotation(
        &self,
        experience_id: &str,
        annotation_id: &str,
    ) -> Result<Option<MapAnnotation>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                annotation_select_sql(
                    "JOIN outdoor_experience_map_annotations c ON c.annotation_id = a.id \
                     WHERE c.outdoor_experience_id = ? AND a.id = ? \
                        AND c.is_deleted = FALSE AND a.is_deleted = FALSE",
                ),
                vec![
                    experience_id.to_owned().into(),
                    annotation_id.to_owned().into(),
                ],
            ))
            .await?;
        row.as_ref().map(map_annotation).transpose()
    }

    async fn annotation(&self, annotation_id: &str) -> Result<Option<MapAnnotation>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                annotation_select_sql("WHERE a.id = ? AND a.is_deleted = FALSE"),
                vec![annotation_id.to_owned().into()],
            ))
            .await?;
        row.as_ref().map(map_annotation).transpose()
    }

    async fn update_annotation(
        &self,
        annotation: MapAnnotation,
        patch: &MapAnnotationPatch,
        base_versions: FieldVersions,
        force_fields: BTreeSet<String>,
    ) -> Result<Option<MapAnnotation>, TrailRepositoryError> {
        let touched = patch.touched_fields();
        if touched.is_empty() {
            return Ok(Some(annotation));
        }
        let conflicts = annotation_conflicts(&annotation, &touched, &base_versions, &force_fields);
        if !conflicts.is_empty() {
            return Err(TrailRepositoryError::Conflict(conflicts));
        }
        let mut versions = annotation.field_versions.clone();
        for field in &touched {
            *versions.entry((*field).to_owned()).or_insert(0) += 1;
        }
        let mut assignments = Vec::new();
        let mut values = Vec::<Value>::new();
        if let Some(annotation_type) = &patch.annotation_type {
            assignments.push("annotation_type = ?");
            values.push(annotation_type.clone().into());
        }
        if let Some(title) = &patch.title {
            assignments.push("title = ?");
            values.push(title.clone().into());
        }
        if let Some(note) = &patch.note {
            assignments.push("note = ?");
            values.push(note.clone().into());
        }
        if let Some(elevation_m) = patch.elevation_m {
            assignments.push("elevation_m = ?");
            values.push(elevation_m.into());
        }
        let now = now_rfc3339();
        assignments.push("field_versions_json = ?");
        values.push(json_string(&versions)?.into());
        assignments.push("updated_at = ?");
        values.push(now.into());
        values.push(annotation.id.clone().into());
        let sql = format!(
            "UPDATE map_annotations SET {} WHERE id = ? AND is_deleted = FALSE",
            assignments.join(", ")
        );
        self.db
            .execute(statement(self.db.get_database_backend(), sql, values))
            .await?;
        self.annotation(&annotation.id).await.map_err(Into::into)
    }

    async fn soft_delete_annotation_context(
        &self,
        annotation_id: &str,
        context_table: &str,
        context_column: &str,
        context_id: &str,
        now: &str,
    ) -> Result<bool, TrailRepositoryError> {
        let context_sql = format!(
            "UPDATE {context_table} SET is_deleted = TRUE, updated_at = ? \
             WHERE {context_column} = ? AND annotation_id = ? AND is_deleted = FALSE"
        );
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                context_sql,
                vec![
                    now.to_owned().into(),
                    context_id.to_owned().into(),
                    annotation_id.to_owned().into(),
                ],
            ))
            .await?;
        if result.rows_affected() > 0 {
            self.db
                .execute(statement(
                    self.db.get_database_backend(),
                    "UPDATE map_annotations SET is_deleted = TRUE, updated_at = ? \
                     WHERE id = ? AND is_deleted = FALSE",
                    vec![now.to_owned().into(), annotation_id.to_owned().into()],
                ))
                .await?;
        }
        Ok(result.rows_affected() > 0)
    }
}

fn trail_select_sql(tail: &str) -> String {
    format!(
        "SELECT t.id, t.owner_user_id, t.display_name, t.description, t.source_format, \
                t.original_filename, t.content_type, t.size_bytes, t.sha256_hex, t.bucket, \
                t.object_key, t.normalized_points_json, t.simplified_geojson_json, t.bounds_json, \
                t.distance_m, t.ascent_m, t.descent_m, t.min_elevation_m, t.max_elevation_m, \
                t.start_time, t.end_time, t.point_count, t.is_deleted, t.created_at, t.updated_at \
         {tail}"
    )
}

fn trail_link_select_sql(table: &str, where_clause: &str) -> String {
    trail_select_sql(&format!(
        ", l.linked_by_user_id AS link_linked_by_user_id, l.role AS link_role, \
            l.sort_order AS link_sort_order, l.notes AS link_notes, \
            l.created_at AS link_created_at, l.updated_at AS link_updated_at \
         FROM {table} l \
         JOIN trails t ON t.id = l.trail_id \
         WHERE {where_clause}"
    ))
}

fn annotation_select_sql(tail: &str) -> String {
    format!(
        "SELECT a.id, a.owner_user_id, a.trail_id, a.lng, a.lat, a.elevation_m, \
                a.trail_point_index, a.annotation_type, a.title, a.note, \
                a.field_versions_json, a.is_deleted, a.created_at, a.updated_at \
         FROM map_annotations a {tail}"
    )
}

fn map_trail(row: &QueryResult) -> Result<Trail, DbErr> {
    let source_format: String = row.try_get("", "source_format")?;
    let normalized_points_json: String = row.try_get("", "normalized_points_json")?;
    let simplified_geojson_json: String = row.try_get("", "simplified_geojson_json")?;
    let bounds_json: Option<String> = row.try_get("", "bounds_json")?;
    let normalized_points =
        serde_json::from_str::<Vec<TrailPoint>>(&normalized_points_json).map_err(json_db_error)?;
    let start_elevation_m = normalized_points
        .first()
        .and_then(|point| point.elevation_m);
    let end_elevation_m = normalized_points.last().and_then(|point| point.elevation_m);
    Ok(Trail {
        id: row.try_get("", "id")?,
        owner_user_id: row.try_get("", "owner_user_id")?,
        display_name: row.try_get("", "display_name")?,
        description: row.try_get("", "description")?,
        source_format: TrailSourceFormat::from_key(&source_format).map_err(validation_db_error)?,
        original_filename: row.try_get("", "original_filename")?,
        content_type: row.try_get("", "content_type")?,
        size_bytes: row.try_get("", "size_bytes")?,
        sha256_hex: row.try_get("", "sha256_hex")?,
        bucket: row.try_get("", "bucket")?,
        object_key: row.try_get("", "object_key")?,
        normalized_points,
        simplified_geojson: serde_json::from_str::<JsonValue>(&simplified_geojson_json)
            .map_err(json_db_error)?,
        bounds: bounds_json
            .as_deref()
            .map(serde_json::from_str::<TrailBounds>)
            .transpose()
            .map_err(json_db_error)?,
        distance_m: row.try_get("", "distance_m")?,
        ascent_m: row.try_get("", "ascent_m")?,
        descent_m: row.try_get("", "descent_m")?,
        min_elevation_m: row.try_get("", "min_elevation_m")?,
        max_elevation_m: row.try_get("", "max_elevation_m")?,
        start_elevation_m,
        end_elevation_m,
        start_time: row.try_get("", "start_time")?,
        end_time: row.try_get("", "end_time")?,
        point_count: row.try_get("", "point_count")?,
        is_deleted: row.try_get("", "is_deleted")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

fn map_trail_link(row: &QueryResult) -> Result<MapTrailLink, DbErr> {
    let trail = map_trail(row)?;
    Ok(MapTrailLink {
        trail_id: trail.id.clone(),
        linked_by_user_id: row.try_get("", "link_linked_by_user_id")?,
        role: row.try_get("", "link_role")?,
        sort_order: row.try_get("", "link_sort_order")?,
        notes: row.try_get("", "link_notes")?,
        created_at: row.try_get("", "link_created_at")?,
        updated_at: row.try_get("", "link_updated_at")?,
        trail: TrailSummary::from(&trail),
        simplified_geojson: trail.simplified_geojson,
    })
}

fn map_trip_overview_trail(row: &QueryResult) -> Result<TripOverviewTrail, DbErr> {
    Ok(TripOverviewTrail {
        trip_id: row.try_get("", "overview_trip_id")?,
        trip_title: row.try_get("", "overview_trip_title")?,
        trip_start_date: row.try_get("", "overview_trip_start_date")?,
        trip_end_date: row.try_get("", "overview_trip_end_date")?,
        link: map_trail_link(row)?,
    })
}

fn map_annotation(row: &QueryResult) -> Result<MapAnnotation, DbErr> {
    let versions_json: String = row.try_get("", "field_versions_json")?;
    Ok(MapAnnotation {
        id: row.try_get("", "id")?,
        owner_user_id: row.try_get("", "owner_user_id")?,
        trail_id: row.try_get("", "trail_id")?,
        lng: row.try_get("", "lng")?,
        lat: row.try_get("", "lat")?,
        elevation_m: row.try_get("", "elevation_m")?,
        trail_point_index: row.try_get("", "trail_point_index")?,
        annotation_type: row.try_get("", "annotation_type")?,
        title: row.try_get("", "title")?,
        note: row.try_get("", "note")?,
        field_versions: serde_json::from_str(&versions_json).unwrap_or_default(),
        is_deleted: row.try_get("", "is_deleted")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

fn annotation_conflicts(
    annotation: &MapAnnotation,
    touched_fields: &[&str],
    base_versions: &FieldVersions,
    force_fields: &BTreeSet<String>,
) -> Vec<FieldConflict> {
    touched_fields
        .iter()
        .filter_map(|field| {
            if force_fields.contains(*field) {
                return None;
            }
            let server_version = annotation.field_versions.get(*field).copied().unwrap_or(0);
            let client_version = base_versions.get(*field).copied().unwrap_or(0);
            (server_version != client_version).then(|| FieldConflict {
                field: (*field).to_owned(),
                client_value: JsonValue::Null,
                server_value: annotation_field_value(annotation, field),
                server_version,
            })
        })
        .collect()
}

fn annotation_field_value(annotation: &MapAnnotation, field: &str) -> JsonValue {
    match field {
        "annotation_type" => JsonValue::String(annotation.annotation_type.clone()),
        "title" => annotation
            .title
            .clone()
            .map(JsonValue::String)
            .unwrap_or(JsonValue::Null),
        "note" => annotation
            .note
            .clone()
            .map(JsonValue::String)
            .unwrap_or(JsonValue::Null),
        "elevation_m" => annotation
            .elevation_m
            .and_then(serde_json::Number::from_f64)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        _ => JsonValue::Null,
    }
}

fn initial_versions<const N: usize>(fields: [&str; N]) -> FieldVersions {
    fields
        .into_iter()
        .map(|field| (field.to_owned(), 1))
        .collect::<BTreeMap<_, _>>()
}

fn json_string<T: serde::Serialize>(value: &T) -> Result<String, DbErr> {
    serde_json::to_string(value).map_err(json_db_error)
}

fn optional_json_string<T: serde::Serialize>(value: &Option<T>) -> Result<Option<String>, DbErr> {
    value.as_ref().map(json_string).transpose()
}

fn json_db_error(error: serde_json::Error) -> DbErr {
    DbErr::Custom(format!("invalid trail json: {error}"))
}

fn validation_db_error(error: ValidationError) -> DbErr {
    DbErr::Custom(format!("invalid trail record: {:?}", error.fields))
}
