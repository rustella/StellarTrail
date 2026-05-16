//! 初始用户与会话表 migration，建立微信登录用户、会话 token hash 与基础索引。

use sea_orm_migration::prelude::*;

/// 单个数据库 migration 类型，由 SeaORM migration 框架调用 up/down。
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// 执行 schema 升级逻辑。
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // 初始 migration 使用显式 DDL，确保 SQLite/Postgres/MySQL 初期字段保持一致。
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                wechat_openid TEXT UNIQUE NULL,
                nickname TEXT NULL,
                avatar_url TEXT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id),
                token_hash TEXT NOT NULL UNIQUE,
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                revoked_at TEXT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions(token_hash)",
        )
        .await?;
        Ok(())
    }

    /// 执行 schema 回滚逻辑，尽量撤销 up 中创建的表或索引。
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS sessions")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS users").await?;
        Ok(())
    }
}
