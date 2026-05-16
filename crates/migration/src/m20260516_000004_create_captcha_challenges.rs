//! 图片验证码 challenge migration，建立一次性 ticket、答案摘要、过期和消费状态表。

use sea_orm_migration::prelude::*;

/// 单个数据库 migration 类型，由 SeaORM migration 框架调用 up/down。
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// 执行 schema 升级逻辑。
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // challenge 表只保存答案摘要和消费状态，不保存可直接复用的验证码明文。
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS captcha_challenges (
                id TEXT PRIMARY KEY,
                account TEXT NOT NULL,
                ticket TEXT NOT NULL UNIQUE,
                answer_hash TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                consumed_at TEXT NULL,
                created_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_captcha_challenges_ticket ON captcha_challenges(ticket)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_captcha_challenges_account_created ON captcha_challenges(account, created_at)",
        )
        .await?;
        Ok(())
    }

    /// 执行 schema 回滚逻辑，尽量撤销 up 中创建的表或索引。
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS captcha_challenges")
            .await?;
        Ok(())
    }
}
