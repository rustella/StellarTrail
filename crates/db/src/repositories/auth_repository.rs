//! Authentication repository wrapping persistence for users, sessions, email verification codes, and image captcha challenges.

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr};

use super::statement;
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Iso8601};
use uuid::Uuid;

const USER_SELECT: &str = "id, wechat_openid, username, email, password_hash, nickname, avatar_url, failed_login_attempts, created_at, updated_at";

/// Stable data boundary for `UserRecord`, exposed by or reused within this module.
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

/// Authentication persistence object that centralizes SQL for users, sessions, and verification challenges.
#[derive(Clone)]
pub struct AuthRepository {
    db: DatabaseConnection,
}

impl AuthRepository {
    /// Runs the `new` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Runs the `upsert mock user` server-side flow while preserving input validation, error propagation, and state invariants.
    pub async fn upsert_mock_user(
        &self,
        wechat_openid: &str,
        nickname: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<UserRecord, DbErr> {
        self.upsert_wechat_user(wechat_openid, nickname, avatar_url)
            .await
    }

    /// Runs the `upsert wechat user` server-side flow while preserving input validation, error propagation, and state invariants.
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

    /// Runs the `create password user` server-side flow while preserving input validation, error propagation, and state invariants.
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

    /// Runs the `create session` server-side flow while preserving input validation, error propagation, and state invariants.
    pub async fn create_session(
        &self,
        user_id: &str,
        token_hash: &str,
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
                "INSERT INTO sessions (id, user_id, token_hash, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
                vec![
                    id.clone().into(),
                    user_id.to_owned().into(),
                    token_hash.to_owned().into(),
                    expires_at.into(),
                    now.into(),
                ],
            ))
            .await?;
        Ok(id)
    }

    /// Runs the `create email verification code` server-side flow while preserving input validation, error propagation, and state invariants.
    pub async fn create_email_verification_code(
        &self,
        email: &str,
        purpose: &str,
        code_hash: &str,
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

    /// Runs the `find user by username` server-side flow while preserving input validation, error propagation, and state invariants.
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

    /// Runs the `find user by email` server-side flow while preserving input validation, error propagation, and state invariants.
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

    /// Runs the `find user by openid` server-side flow while preserving input validation, error propagation, and state invariants.
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

    /// Runs the `find user by id` server-side flow while preserving input validation, error propagation, and state invariants.
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

/// Computes a SHA-256 digest for access tokens so the database stores only non-reusable token hashes.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Runs the `map user` server-side flow while preserving input validation, error propagation, and state invariants.
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

/// Runs the `now rfc3339` server-side flow while preserving input validation, error propagation, and state invariants.
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

    /// Runs the `creates session and finds user by token hash` server-side flow while preserving input validation, error propagation, and state invariants.
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
