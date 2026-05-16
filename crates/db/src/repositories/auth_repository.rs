//! 认证 repository，封装用户、会话、邮箱验证码和图片验证码 challenge 的持久化访问。

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr};

use super::statement;
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Iso8601};
use uuid::Uuid;

const USER_SELECT: &str = "id, wechat_openid, username, email, password_hash, nickname, avatar_url, failed_login_attempts, created_at, updated_at";

/// UserRecord 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
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

/// 认证持久化对象，集中封装用户、会话、验证码相关 SQL。
#[derive(Clone)]
pub struct AuthRepository {
    db: DatabaseConnection,
}

impl AuthRepository {
    /// 执行 `new` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 执行 `upsert mock user` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub async fn upsert_mock_user(
        &self,
        wechat_openid: &str,
        nickname: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<UserRecord, DbErr> {
        self.upsert_wechat_user(wechat_openid, nickname, avatar_url)
            .await
    }

    /// 执行 `upsert wechat user` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub async fn upsert_wechat_user(
        &self,
        wechat_openid: &str,
        nickname: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<UserRecord, DbErr> {
        // openid 是微信用户稳定标识；已存在用户只更新展示资料，不新建账号。
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

    /// 执行 `create password user` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 执行 `create session` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 执行 `create email verification code` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 按邮箱、用途和验证码摘要查找未过期记录，并原子标记为已消费。
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
        // 找不到可消费验证码时直接返回 false，由服务层映射为校验错误。
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

    /// 创建一次性图片验证码 challenge，并在本地环境返回 debug 答案。
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

    /// 校验并消费图片验证码 challenge，确保 ticket 不能重复使用。
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

    /// 按会话 token hash 查找有效用户，同时过滤撤销、过期和已删除数据。
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

    /// 按规范化账号查找用户，支持用户名或邮箱登录。
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

    /// 执行 `find user by username` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 执行 `find user by email` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 记录一次密码登录失败并递增失败计数。
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

    /// 登录成功后重置失败计数和失败时间。
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

    /// 执行 `find user by openid` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 执行 `find user by id` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

/// 对访问 token 做 SHA-256 摘要，数据库只保存不可直接使用的 token hash。
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// 执行 `map user` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

/// 执行 `now rfc3339` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 执行 `creates session and finds user by token hash` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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
