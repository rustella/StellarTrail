//! Account-scoped outdoor skill favorite persistence.
//!
//! The current catalog only contains DB-backed knots, but route and response
//! names intentionally use "skill favorites" so future skill categories can be
//! added without changing the user-facing API namespace.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr};

use super::statement;

/// Stored favorite status for one knot and one user.
#[derive(Clone, Debug)]
pub struct KnotFavoriteStatus {
    pub knot_id: String,
    pub is_favorited: bool,
    pub favorited_at: Option<String>,
}

/// One active favorite row used by list responses.
#[derive(Clone, Debug)]
pub struct KnotFavoriteListEntry {
    pub knot_id: String,
    pub favorited_at: String,
}

/// Counts used to build the favorites filter bar.
#[derive(Clone, Debug, Default)]
pub struct SkillFavoriteCounts {
    pub total_count: u32,
    pub knot_count: u32,
}

/// Persistence object for user skill favorites.
#[derive(Clone)]
pub struct SkillFavoriteRepository {
    db: DatabaseConnection,
}

impl SkillFavoriteRepository {
    /// Creates a repository using the shared application database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Returns whether a knot exists in the public catalog.
    pub async fn knot_exists(&self, knot_id: &str) -> Result<bool, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT id FROM knots WHERE id = ? LIMIT 1",
                vec![knot_id.to_owned().into()],
            ))
            .await?;
        Ok(row.is_some())
    }

    /// Lists active favorite knots for one user.
    pub async fn list_knot_favorites(
        &self,
        user_id: &str,
        offset: u32,
        limit: u32,
    ) -> Result<(Vec<KnotFavoriteListEntry>, u32, Option<u32>), DbErr> {
        let total_count = self.active_knot_count(user_id).await?;
        let rows = self
            .db
            .query_all(statement(
                self.db.get_database_backend(),
                "SELECT knot_id, favorited_at FROM user_knot_favorites \
                 WHERE user_id = ? AND is_deleted = FALSE \
                 ORDER BY favorited_at DESC, knot_id ASC LIMIT ? OFFSET ?",
                vec![
                    user_id.to_owned().into(),
                    (limit as i64).into(),
                    (offset as i64).into(),
                ],
            ))
            .await?;
        let items = rows
            .into_iter()
            .map(|row| {
                Ok(KnotFavoriteListEntry {
                    knot_id: row.try_get("", "knot_id")?,
                    favorited_at: row.try_get("", "favorited_at")?,
                })
            })
            .collect::<Result<Vec<_>, DbErr>>()?;
        let end = offset.saturating_add(items.len() as u32);
        let next_offset = if end < total_count { Some(end) } else { None };
        Ok((items, total_count, next_offset))
    }

    /// Returns active favorite counts by skill category.
    pub async fn counts(&self, user_id: &str) -> Result<SkillFavoriteCounts, DbErr> {
        let knot_count = self.active_knot_count(user_id).await?;
        Ok(SkillFavoriteCounts {
            total_count: knot_count,
            knot_count,
        })
    }

    /// Returns the favorite status for one knot without modifying it.
    pub async fn knot_status(
        &self,
        user_id: &str,
        knot_id: &str,
    ) -> Result<KnotFavoriteStatus, DbErr> {
        Ok(self
            .stored_knot_status(user_id, knot_id)
            .await?
            .unwrap_or_else(|| inactive_status(knot_id)))
    }

    /// Idempotently favorites a knot for a user.
    pub async fn favorite_knot(
        &self,
        user_id: &str,
        knot_id: &str,
    ) -> Result<KnotFavoriteStatus, DbErr> {
        if let Some(status) = self.stored_knot_status(user_id, knot_id).await? {
            if status.is_favorited {
                return Ok(status);
            }
            self.db
                .execute(statement(
                    self.db.get_database_backend(),
                    "UPDATE user_knot_favorites \
                     SET is_deleted = FALSE, favorited_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP \
                     WHERE user_id = ? AND knot_id = ?",
                    vec![user_id.to_owned().into(), knot_id.to_owned().into()],
                ))
                .await?;
            return self.knot_status(user_id, knot_id).await;
        }

        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO user_knot_favorites (
                    user_id, knot_id, is_deleted, favorited_at, created_at, updated_at
                 ) VALUES (
                    ?, ?, FALSE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
                 )",
                vec![user_id.to_owned().into(), knot_id.to_owned().into()],
            ))
            .await?;
        self.knot_status(user_id, knot_id).await
    }

    /// Idempotently soft-deletes a user's knot favorite.
    pub async fn unfavorite_knot(
        &self,
        user_id: &str,
        knot_id: &str,
    ) -> Result<KnotFavoriteStatus, DbErr> {
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE user_knot_favorites \
                 SET is_deleted = TRUE, updated_at = CURRENT_TIMESTAMP \
                 WHERE user_id = ? AND knot_id = ?",
                vec![user_id.to_owned().into(), knot_id.to_owned().into()],
            ))
            .await?;
        self.knot_status(user_id, knot_id).await
    }

    async fn active_knot_count(&self, user_id: &str) -> Result<u32, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT COUNT(*) AS count FROM user_knot_favorites \
                 WHERE user_id = ? AND is_deleted = FALSE",
                vec![user_id.to_owned().into()],
            ))
            .await?;
        let count = row
            .map(|row| row.try_get::<i64>("", "count"))
            .transpose()?
            .unwrap_or_default();
        Ok(count.max(0) as u32)
    }

    async fn stored_knot_status(
        &self,
        user_id: &str,
        knot_id: &str,
    ) -> Result<Option<KnotFavoriteStatus>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                "SELECT knot_id, is_deleted, favorited_at FROM user_knot_favorites \
                 WHERE user_id = ? AND knot_id = ? LIMIT 1",
                vec![user_id.to_owned().into(), knot_id.to_owned().into()],
            ))
            .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let is_deleted: bool = row.try_get("", "is_deleted")?;
        Ok(Some(KnotFavoriteStatus {
            knot_id: row.try_get("", "knot_id")?,
            is_favorited: !is_deleted,
            favorited_at: if is_deleted {
                None
            } else {
                Some(row.try_get("", "favorited_at")?)
            },
        }))
    }
}

fn inactive_status(knot_id: &str) -> KnotFavoriteStatus {
    KnotFavoriteStatus {
        knot_id: knot_id.to_owned(),
        is_favorited: false,
        favorited_at: None,
    }
}
