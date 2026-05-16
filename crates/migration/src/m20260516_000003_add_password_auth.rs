//! 密码登录 migration，为用户表增加用户名、邮箱、密码摘要、失败计数和邮箱验证码表。

use sea_orm_migration::prelude::*;

/// 单个数据库 migration 类型，由 SeaORM migration 框架调用 up/down。
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// 执行 schema 升级逻辑。
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // 密码登录以兼容方式扩展 users 表，旧微信用户可继续保留空用户名/邮箱。
        db.execute_unprepared("ALTER TABLE users ADD COLUMN username TEXT NULL")
            .await?;
        db.execute_unprepared("ALTER TABLE users ADD COLUMN email TEXT NULL")
            .await?;
        db.execute_unprepared("ALTER TABLE users ADD COLUMN password_hash TEXT NULL")
            .await?;
        db.execute_unprepared(
            "ALTER TABLE users ADD COLUMN failed_login_attempts INTEGER NOT NULL DEFAULT 0",
        )
        .await?;
        db.execute_unprepared("ALTER TABLE users ADD COLUMN last_failed_login_at TEXT NULL")
            .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users(username) WHERE username IS NOT NULL",
        )
        .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email) WHERE email IS NOT NULL",
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS email_verification_codes (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL,
                purpose TEXT NOT NULL,
                code_hash TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                consumed_at TEXT NULL,
                created_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_email_verification_email_purpose ON email_verification_codes(email, purpose, created_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_email_verification_code_hash ON email_verification_codes(code_hash)",
        )
        .await?;
        Ok(())
    }

    /// 执行 schema 回滚逻辑，尽量撤销 up 中创建的表或索引。
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS email_verification_codes")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_users_email")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_users_username")
            .await?;
        Ok(())
    }
}
