use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Statement};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Iso8601};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct UserRecord {
    pub id: String,
    pub wechat_openid: Option<String>,
    pub nickname: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct AuthRepository {
    db: DatabaseConnection,
}

impl AuthRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn upsert_mock_user(
        &self,
        wechat_openid: &str,
        nickname: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<UserRecord, DbErr> {
        if let Some(user) = self.find_user_by_openid(wechat_openid).await? {
            let now = now_rfc3339();
            self.db
                .execute(Statement::from_sql_and_values(
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
            nickname,
            avatar_url,
            created_at: now.clone(),
            updated_at: now,
        };
        self.db
            .execute(Statement::from_sql_and_values(
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
            .execute(Statement::from_sql_and_values(
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

    pub async fn find_user_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<UserRecord>, DbErr> {
        let now = now_rfc3339();
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                self.db.get_database_backend(),
                r#"SELECT users.id, users.wechat_openid, users.nickname, users.avatar_url, users.created_at, users.updated_at
                   FROM sessions
                   JOIN users ON users.id = sessions.user_id
                   WHERE sessions.token_hash = ?
                     AND sessions.revoked_at IS NULL
                     AND sessions.expires_at > ?
                     AND users.deleted_at IS NULL
                   LIMIT 1"#,
                vec![token_hash.to_owned().into(), now.into()],
            ))
            .await?;
        row.map(|row| map_user(&row)).transpose()
    }

    async fn find_user_by_openid(&self, wechat_openid: &str) -> Result<Option<UserRecord>, DbErr> {
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                self.db.get_database_backend(),
                "SELECT id, wechat_openid, nickname, avatar_url, created_at, updated_at FROM users WHERE wechat_openid = ? AND deleted_at IS NULL LIMIT 1",
                vec![wechat_openid.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_user(&row)).transpose()
    }

    async fn find_user_by_id(&self, user_id: &str) -> Result<Option<UserRecord>, DbErr> {
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(
                self.db.get_database_backend(),
                "SELECT id, wechat_openid, nickname, avatar_url, created_at, updated_at FROM users WHERE id = ? AND deleted_at IS NULL LIMIT 1",
                vec![user_id.to_owned().into()],
            ))
            .await?;
        row.map(|row| map_user(&row)).transpose()
    }
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn map_user(row: &sea_orm::QueryResult) -> Result<UserRecord, DbErr> {
    Ok(UserRecord {
        id: row.try_get("", "id")?,
        wechat_openid: row.try_get("", "wechat_openid")?,
        nickname: row.try_get("", "nickname")?,
        avatar_url: row.try_get("", "avatar_url")?,
        created_at: row.try_get("", "created_at")?,
        updated_at: row.try_get("", "updated_at")?,
    })
}

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
