//! Adds internal commit tracking to client version records and seeds the WeChat 0.2.1 release.

use sea_orm_migration::prelude::*;

/// Single database migration type invoked by the SeaORM migration framework for up/down operations.
#[derive(DeriveMigrationName)]
pub struct Migration;

const WECHAT_021_COMMIT_HASH: &str = "376fd6c1ef08636477d5257ab720bc783beeb358";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Runs the schema upgrade logic.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("ALTER TABLE client_versions ADD COLUMN commit_hash TEXT NULL")
            .await?;
        db.execute_unprepared(&format!(
            r#"INSERT INTO client_versions (
                id, client_key, version, title, release_notes_json, status,
                commit_hash, published_at, created_at, updated_at
            ) VALUES (
                'wechat-miniprogram-0.2.1',
                'wechat_miniprogram',
                '0.2.1',
                '0.2.1 离线绳结访问修复',
                '{{"feature":["离线时保留有效登录态，已缓存绳结列表和详情可以继续查看。"],"bug_fix":["修复弱网或离线刷新登录态失败时误退出登录。","修复离线打开已缓存绳结时仍被协议确认流程拦住的问题。"]}}',
                'published',
                '{WECHAT_021_COMMIT_HASH}',
                '2026-05-31T00:00:00Z',
                '2026-05-31T00:00:00Z',
                '2026-05-31T00:00:00Z'
            ) ON CONFLICT(client_key, version) DO UPDATE SET
                title = excluded.title,
                release_notes_json = excluded.release_notes_json,
                status = excluded.status,
                commit_hash = excluded.commit_hash,
                published_at = excluded.published_at,
                updated_at = excluded.updated_at"#
        ))
        .await?;
        Ok(())
    }

    /// Runs schema rollback logic and tries to undo rows or columns created by up.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "DELETE FROM client_versions WHERE client_key = 'wechat_miniprogram' AND version = '0.2.1'",
        )
        .await?;
        db.execute_unprepared("ALTER TABLE client_versions DROP COLUMN commit_hash")
            .await?;
        Ok(())
    }
}
