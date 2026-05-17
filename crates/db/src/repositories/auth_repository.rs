//! Persistence boundary for StellarTrail authentication data.
//!
//! This repository centralizes raw SQL for users, opaque access-token sessions,
//! refresh-token rotation, email verification codes, and captcha challenges. The
//! service layer passes only token hashes into this module; plaintext access and
//! refresh tokens are never stored in the database. Keeping all session queries
//! here also makes revocation, expiry, and deleted-user checks auditable in one
//! place.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr};

use super::statement;
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Iso8601};
use uuid::Uuid;

const USER_SELECT: &str = "id, wechat_openid, username, email, password_hash, nickname, avatar_url, failed_login_attempts, created_at, updated_at";

/// Database projection for an application user returned by authentication queries.
#[derive(Clone, Debug)]
pub struct UserRecord {
    pub id: String,
    pub wechat_openid: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub failed_login_attempts: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Minimal session projection needed after a refresh-token lookup.
///
/// The lookup returns the session id so the caller can rotate exactly that row,
/// and it returns a user snapshot so the response can be built without a second
/// database round trip.
#[derive(Clone, Debug)]
pub struct RefreshSessionRecord {
    pub session_id: String,
    pub user: UserRecord,
}

/// Repository wrapper around a SeaORM connection for authentication storage.
///
/// Methods in this type deliberately use parameterized statements instead of
/// interpolating user input, because most call sites handle credentials or
/// account identifiers supplied directly by clients.
#[derive(Clone)]
pub struct AuthRepository {
    db: DatabaseConnection,
}

impl AuthRepository {
    /// Creates a repository bound to the provided database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Creates or updates a local mock WeChat user for development-only login.
    ///
    /// This delegates to the normal WeChat upsert path so tests and local smoke
    /// flows exercise the same user/session schema as real code2session login.
    pub async fn upsert_mock_user(
        &self,
        wechat_openid: &str,
        nickname: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<UserRecord, DbErr> {
        self.upsert_wechat_user(wechat_openid, nickname, avatar_url)
            .await
    }

    /// Creates a user for a WeChat `openid` or updates the existing display profile.
    ///
    /// The `openid` is treated as the stable identity key. When a user already
    /// exists, only non-security profile fields are updated, preserving existing
    /// sessions, password credentials, and audit timestamps.
    pub async fn upsert_wechat_user(
        &self,
        wechat_openid: &str,
        nickname: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<UserRecord, DbErr> {
        // openid is the stable WeChat user identifier; existing users only update display fields instead of creating a new account.
        if let Some(user) = self.find_user_by_openid(wechat_openid).await? {
            let now = now_rfc3339();
            self.db
                .execute(statement(
                    self.db.get_database_backend(),
                    "UPDATE users SET nickname = ?, avatar_url = ?, updated_at = ? WHERE id = ?",
                    vec![
                        nickname.into(),
                        avatar_url.into(),
                        now.into(),
                        user.id.clone().into(),
                    ],
                ))
                .await?;
            return self
                .find_user_by_id(&user.id)
                .await?
                .ok_or_else(|| DbErr::Custom("updated user not found".to_owned()));
        }

        let now = now_rfc3339();
        let user = UserRecord {
            id: Uuid::new_v4().to_string(),
            wechat_openid: Some(wechat_openid.to_owned()),
            username: None,
            email: None,
            password_hash: None,
            nickname,
            avatar_url,
            failed_login_attempts: 0,
            created_at: now.clone(),
            updated_at: now,
        };
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                "INSERT INTO users (id, wechat_openid, nickname, avatar_url, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                vec![
                    user.id.clone().into(),
                    user.wechat_openid.clone().into(),
                    user.nickname.clone().into(),
                    user.avatar_url.clone().into(),
                    user.created_at.clone().into(),
                    user.updated_at.clone().into(),
                ],
            ))
            .await?;
        Ok(user)
    }

    /// Inserts a password-based account after service-layer validation succeeds.
    ///
    /// The caller is responsible for hashing the password before it reaches the
    /// repository so this function never sees or persists plaintext passwords.
    pub async fn create_password_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<UserRecord, DbErr> {
        let now = now_rfc3339();
        let id = Uuid::new_v4().to_string();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO users (
                    id, username, email, password_hash, nickname, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?)"#,
                vec![
                    id.clone().into(),
                    username.to_owned().into(),
                    email.to_owned().into(),
                    password_hash.to_owned().into(),
                    username.to_owned().into(),
                    now.clone().into(),
                    now.into(),
                ],
            ))
            .await?;
        self.find_user_by_id(&id)
            .await?
            .ok_or_else(|| DbErr::Custom("created user not found".to_owned()))
    }

    /// Persists a new opaque access-token session with its paired refresh token hash.
    ///
    /// Both token arguments must already be SHA-256 digests of client-visible
    /// random tokens. The database stores the access expiry and refresh expiry
    /// separately so a short-lived access token can be renewed until the longer
    /// refresh window closes or the session is revoked.
    pub async fn create_session(
        &self,
        user_id: &str,
        token_hash: &str,
        expires_at: OffsetDateTime,
        refresh_token_hash: &str,
        refresh_expires_at: OffsetDateTime,
    ) -> Result<String, DbErr> {
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        // Persist timestamps as RFC3339 strings to match the existing session
        // schema and keep SQLite/PostgreSQL behavior consistent in tests.
        let expires_at = expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(|err| DbErr::Custom(err.to_string()))?;
        let refresh_expires_at = refresh_expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(|err| DbErr::Custom(err.to_string()))?;
        // Insert both token hashes in the same row so revoking a session disables
        // the access token and the refresh token together.
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO sessions (
                    id, user_id, token_hash, refresh_token_hash, expires_at, refresh_expires_at, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?)"#,
                vec![
                    id.clone().into(),
                    user_id.to_owned().into(),
                    token_hash.to_owned().into(),
                    refresh_token_hash.to_owned().into(),
                    expires_at.into(),
                    refresh_expires_at.into(),
                    now.into(),
                ],
            ))
            .await?;
        Ok(id)
    }

    /// Stores a hashed email verification code for a specific account action.
    ///
    /// The plaintext code is returned only to the service layer in local test
    /// environments; this repository stores a digest so leaked rows cannot be
    /// used to complete registration.
    pub async fn create_email_verification_code(
        &self,
        email: &str,
        purpose: &str,
        code_hash: &str,
        expires_at: OffsetDateTime,
    ) -> Result<String, DbErr> {
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        // The update statement below is the replay guard: it succeeds only while
        // the stored refresh hash still matches the hash presented by the client.
        let expires_at = expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(|err| DbErr::Custom(err.to_string()))?;
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO email_verification_codes (
                    id, email, purpose, code_hash, expires_at, created_at
                ) VALUES (?, ?, ?, ?, ?, ?)"#,
                vec![
                    id.clone().into(),
                    email.to_owned().into(),
                    purpose.to_owned().into(),
                    code_hash.to_owned().into(),
                    expires_at.into(),
                    now.into(),
                ],
            ))
            .await?;
        Ok(id)
    }

    /// Finds an unexpired record by email, purpose, and code digest, then atomically marks it as consumed.
    pub async fn consume_email_verification_code(
        &self,
        email: &str,
        purpose: &str,
        code_hash: &str,
    ) -> Result<bool, DbErr> {
        let now = now_rfc3339();
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                r#"SELECT id FROM email_verification_codes
                   WHERE email = ?
                     AND purpose = ?
                     AND code_hash = ?
                     AND consumed_at IS NULL
                     AND expires_at > ?
                   ORDER BY created_at DESC
                   LIMIT 1"#,
                vec![
                    email.to_owned().into(),
                    purpose.to_owned().into(),
                    code_hash.to_owned().into(),
                    now.clone().into(),
                ],
            ))
            .await?;
        // Return false when no consumable code exists; the service layer maps that to a validation error.
        let Some(row) = row else {
            return Ok(false);
        };
        let id: String = row.try_get("", "id")?;
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE email_verification_codes SET consumed_at = ? WHERE id = ? AND consumed_at IS NULL",
                vec![now.into(), id.into()],
            ))
            .await?;
        // A zero-row update means the refresh token was expired, revoked,
        // deleted-user-bound, or already rotated by another request.
        Ok(result.rows_affected() > 0)
    }

    /// Creates a one-time image captcha challenge and returns the debug answer in the local environment.
    pub async fn create_captcha_challenge(
        &self,
        account: &str,
        ticket: &str,
        answer_hash: &str,
        expires_at: OffsetDateTime,
    ) -> Result<String, DbErr> {
        let id = Uuid::new_v4().to_string();
        let now = now_rfc3339();
        let expires_at = expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(|err| DbErr::Custom(err.to_string()))?;
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"INSERT INTO captcha_challenges (
                    id, account, ticket, answer_hash, expires_at, created_at
                ) VALUES (?, ?, ?, ?, ?, ?)"#,
                vec![
                    id.clone().into(),
                    account.to_owned().into(),
                    ticket.to_owned().into(),
                    answer_hash.to_owned().into(),
                    expires_at.into(),
                    now.into(),
                ],
            ))
            .await?;
        Ok(id)
    }

    /// Validates and consumes an image captcha challenge so the ticket cannot be reused.
    pub async fn consume_captcha_challenge(
        &self,
        ticket: &str,
        answer_hash: &str,
    ) -> Result<bool, DbErr> {
        let now = now_rfc3339();
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                r#"SELECT id FROM captcha_challenges
                   WHERE ticket = ?
                     AND answer_hash = ?
                     AND consumed_at IS NULL
                     AND expires_at > ?
                   LIMIT 1"#,
                vec![
                    ticket.to_owned().into(),
                    answer_hash.to_owned().into(),
                    now.clone().into(),
                ],
            ))
            .await?;
        let Some(row) = row else {
            return Ok(false);
        };
        let id: String = row.try_get("", "id")?;
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                "UPDATE captcha_challenges SET consumed_at = ? WHERE id = ? AND consumed_at IS NULL",
                vec![now.into(), id.into()],
            ))
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Finds the active user by session token hash while filtering revoked, expired, and deleted data.
    pub async fn find_user_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<UserRecord>, DbErr> {
        let now = now_rfc3339();
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    r#"SELECT users.{user_select}
                       FROM sessions
                       JOIN users ON users.id = sessions.user_id
                       WHERE sessions.token_hash = ?
                         AND sessions.revoked_at IS NULL
                         AND sessions.expires_at > ?
                         AND users.deleted_at IS NULL
                       LIMIT 1"#,
                    user_select = USER_SELECT.replace(", ", ", users."),
                ),
                vec![token_hash.to_owned().into(), now.into()],
            ))
            .await?;
        row.map(|row| map_user(&row)).transpose()
    }

    /// Looks up a refresh-token hash that is still eligible for rotation.
    ///
    /// Refresh lookup intentionally ignores the access-token expiry because the
    /// purpose of this query is to recover from an expired access token. It still
    /// rejects revoked sessions, expired refresh windows, and deleted users.
    pub async fn find_session_by_refresh_token_hash(
        &self,
        refresh_token_hash: &str,
    ) -> Result<Option<RefreshSessionRecord>, DbErr> {
        let now = now_rfc3339();
        // Refresh-token lookup uses the refresh hash and refresh expiry only;
        // the access token may already be expired when clients call this path.
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    r#"SELECT sessions.id AS session_id, users.{user_select}
                       FROM sessions
                       JOIN users ON users.id = sessions.user_id
                       WHERE sessions.refresh_token_hash = ?
                         AND sessions.revoked_at IS NULL
                         AND sessions.refresh_expires_at > ?
                         AND users.deleted_at IS NULL
                       LIMIT 1"#,
                    user_select = USER_SELECT.replace(", ", ", users."),
                ),
                vec![refresh_token_hash.to_owned().into(), now.into()],
            ))
            .await?;
        row.map(|row| {
            Ok(RefreshSessionRecord {
                session_id: row.try_get("", "session_id")?,
                user: map_user(&row)?,
            })
        })
        .transpose()
    }

    /// Atomically replaces the access and refresh token hashes for one active session.
    ///
    /// The `old_refresh_token_hash` appears in the `WHERE` clause, so only the
    /// first request using a given refresh token can update the row. Concurrent
    /// or replayed refresh calls affect zero rows and are treated as unauthorized
    /// by the service layer.
    pub async fn rotate_session_tokens(
        &self,
        session_id: &str,
        old_refresh_token_hash: &str,
        token_hash: &str,
        expires_at: OffsetDateTime,
        refresh_token_hash: &str,
        refresh_expires_at: OffsetDateTime,
    ) -> Result<bool, DbErr> {
        let now = now_rfc3339();
        // The update statement below is the replay guard: it succeeds only while
        // the stored refresh hash still matches the hash presented by the client.
        let expires_at = expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(|err| DbErr::Custom(err.to_string()))?;
        let refresh_expires_at = refresh_expires_at
            .format(&Iso8601::DEFAULT)
            .map_err(|err| DbErr::Custom(err.to_string()))?;
        // Rotate the access and refresh token hashes in one write so clients
        // never observe a new access token paired with an old refresh token.
        let result = self
            .db
            .execute(statement(
                self.db.get_database_backend(),
                r#"UPDATE sessions
                   SET token_hash = ?,
                       refresh_token_hash = ?,
                       expires_at = ?,
                       refresh_expires_at = ?,
                       refreshed_at = ?
                   WHERE id = ?
                     AND refresh_token_hash = ?
                     AND refresh_expires_at > ?
                     AND revoked_at IS NULL
                     AND EXISTS (
                         SELECT 1 FROM users
                         WHERE users.id = sessions.user_id
                           AND users.deleted_at IS NULL
                     )"#,
                vec![
                    token_hash.to_owned().into(),
                    refresh_token_hash.to_owned().into(),
                    expires_at.into(),
                    refresh_expires_at.into(),
                    now.clone().into(),
                    session_id.to_owned().into(),
                    old_refresh_token_hash.to_owned().into(),
                    now.into(),
                ],
            ))
            .await?;
        // A zero-row update means the refresh token was expired, revoked,
        // deleted-user-bound, or already rotated by another request.
        Ok(result.rows_affected() > 0)
    }

    /// Finds a user by normalized account identifier, supporting username or email login.
    pub async fn find_user_by_login_account(
        &self,
        account: &str,
    ) -> Result<Option<UserRecord>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "SELECT {USER_SELECT} FROM users WHERE (username = ? OR email = ?) AND deleted_at IS NULL LIMIT 1"
                ),
                vec![account.to_owned().into(), account.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_user(&row)).transpose()
    }

    /// Finds a non-deleted user by normalized username for password login and uniqueness checks.
    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<UserRecord>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!("SELECT {USER_SELECT} FROM users WHERE username = ? AND deleted_at IS NULL LIMIT 1"),
                vec![username.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_user(&row)).transpose()
    }

    /// Finds a non-deleted user by normalized email for password login and registration checks.
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<UserRecord>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "SELECT {USER_SELECT} FROM users WHERE email = ? AND deleted_at IS NULL LIMIT 1"
                ),
                vec![email.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_user(&row)).transpose()
    }

    /// Records one failed password login and increments the failure counter.
    pub async fn record_failed_password_login(&self, user_id: &str) -> Result<(), DbErr> {
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"UPDATE users
                   SET failed_login_attempts = failed_login_attempts + 1,
                       last_failed_login_at = ?,
                       updated_at = ?
                   WHERE id = ?"#,
                vec![now.clone().into(), now.into(), user_id.to_owned().into()],
            ))
            .await?;
        Ok(())
    }

    /// Resets the failure count and failure timestamp after a successful login.
    pub async fn reset_failed_password_login(&self, user_id: &str) -> Result<(), DbErr> {
        let now = now_rfc3339();
        self.db
            .execute(statement(
                self.db.get_database_backend(),
                r#"UPDATE users
                   SET failed_login_attempts = 0,
                       last_failed_login_at = NULL,
                       updated_at = ?
                   WHERE id = ?"#,
                vec![now.into(), user_id.to_owned().into()],
            ))
            .await?;
        Ok(())
    }

    /// Finds a non-deleted user by WeChat `openid` during code2session login.
    async fn find_user_by_openid(&self, wechat_openid: &str) -> Result<Option<UserRecord>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!("SELECT {USER_SELECT} FROM users WHERE wechat_openid = ? AND deleted_at IS NULL LIMIT 1"),
                vec![wechat_openid.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_user(&row)).transpose()
    }

    /// Reloads a non-deleted user by primary key after an insert or profile update.
    async fn find_user_by_id(&self, user_id: &str) -> Result<Option<UserRecord>, DbErr> {
        let row = self
            .db
            .query_one(statement(
                self.db.get_database_backend(),
                format!(
                    "SELECT {USER_SELECT} FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1"
                ),
                vec![user_id.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_user(&row)).transpose()
    }
}

/// Computes a SHA-256 digest for opaque access or refresh tokens before persistence.
///
/// The digest is deterministic for lookups but the original bearer credential is
/// not recoverable from the stored value, which limits the blast radius of a
/// database leak.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Converts a SQL row containing `USER_SELECT` columns into the repository user projection.
///
/// All authentication queries use the same projection so response construction
/// stays consistent across WeChat login, password login, access-token lookup,
/// and refresh-token lookup.
fn map_user(row: &sea_orm::QueryResult) -> Result<UserRecord, DbErr> {
    Ok(UserRecord {
        id: row.try_get("", "id")?,
        wechat_openid: row.try_get("", "wechat_openid")?,
        username: row.try_get("", "username")?,
        email: row.try_get("", "email")?,
        password_hash: row.try_get("", "password_hash")?,
        nickname: row.try_get("", "nickname")?,
        avatar_url: row.try_get("", "avatar_url")?,
        failed_login_attempts: row.try_get("", "failed_login_attempts")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

/// Formats the current UTC time in the same RFC3339 representation stored by session tables.
fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Iso8601::DEFAULT)
        .expect("RFC3339 timestamp formatting should be infallible")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm_migration::prelude::MigratorTrait;
    use stellartrail_migration::Migrator;

    /// Verifies that a newly created access-token session can be resolved back to its user.
    #[tokio::test]
    async fn creates_session_and_finds_user_by_token_hash() {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();
        let repo = AuthRepository::new(db);
        let user = repo
            .upsert_mock_user("mock:test", Some("测试".to_owned()), None)
            .await
            .unwrap();
        let token_hash = hash_token("plain-token");
        repo.create_session(
            &user.id,
            &token_hash,
            OffsetDateTime::now_utc() + time::Duration::hours(2),
            &hash_token("plain-refresh-token"),
            OffsetDateTime::now_utc() + time::Duration::days(30),
        )
        .await
        .unwrap();

        let found = repo
            .find_user_by_token_hash(&token_hash)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(found.id, user.id);
        assert_eq!(found.nickname.as_deref(), Some("测试"));
    }
}
