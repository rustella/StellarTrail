//! Creates database-backed client release notes for public clients and admin maintenance.

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
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS client_versions (
                id TEXT PRIMARY KEY,
                client_key TEXT NOT NULL CHECK (client_key IN ('wechat_miniprogram', 'web', 'android', 'ios', 'macos')),
                version TEXT NOT NULL,
                title TEXT NOT NULL,
                release_notes_json TEXT NOT NULL DEFAULT '[]',
                commit_hash TEXT NULL,
                status TEXT NOT NULL CHECK (status IN ('draft', 'published')),
                published_at TEXT NULL,
                created_by_user_id TEXT NULL REFERENCES users(id) ON DELETE SET NULL,
                updated_by_user_id TEXT NULL REFERENCES users(id) ON DELETE SET NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(client_key, version)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_client_versions_public ON client_versions(client_key, status, published_at)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_client_versions_admin ON client_versions(client_key, status, updated_at)",
        )
        .await?;
        db.execute_unprepared(
            r#"INSERT INTO client_versions (
                id, client_key, version, title, release_notes_json, status,
                published_at, created_at, updated_at
            ) VALUES (
                'wechat-miniprogram-0.1.0',
                'wechat_miniprogram',
                '0.1.0',
                '0.1.0 初始版本',
                '{"feature":["新增个人装备库，记录装备分类、重量、价格、存放位置和标签。","新增户外技能与绳结教学，支持绳结详情、动图和离线缓存。","新增装备图鉴，浏览已审核收录的市面装备并支持投稿。","新增账户资料、头像昵称、邮箱绑定、黑夜模式和意见反馈入口。"],"bug_fix":[]}',
                'published',
                '2026-05-23T00:00:00Z',
                '2026-05-23T00:00:00Z',
                '2026-05-23T00:00:00Z'
            ) ON CONFLICT(client_key, version) DO NOTHING"#,
        )
        .await?;
        db.execute_unprepared(&format!(
            r#"INSERT INTO client_versions (
                id, client_key, version, title, release_notes_json, commit_hash, status,
                published_at, created_at, updated_at
            ) VALUES (
                'wechat-miniprogram-0.2.1',
                'wechat_miniprogram',
                '0.2.1',
                '0.2.1 离线绳结访问修复',
                '{{"feature":["离线时保留有效登录态，已缓存绳结列表和详情可以继续查看。"],"bug_fix":["修复弱网或离线刷新登录态失败时误退出登录。","修复离线打开已缓存绳结时仍被协议确认流程拦住的问题。"]}}',
                '{WECHAT_021_COMMIT_HASH}',
                'published',
                '2026-05-31T00:00:00Z',
                '2026-05-31T00:00:00Z',
                '2026-05-31T00:00:00Z'
            ) ON CONFLICT(client_key, version) DO UPDATE SET
                title = excluded.title,
                release_notes_json = excluded.release_notes_json,
                commit_hash = excluded.commit_hash,
                status = excluded.status,
                published_at = excluded.published_at,
                updated_at = excluded.updated_at"#
        ))
        .await?;
        db.execute_unprepared(
            r#"INSERT INTO client_versions (
                id, client_key, version, title, release_notes_json, commit_hash, status,
                published_at, created_at, updated_at
            ) VALUES (
                'android-0-0-1',
                'android',
                '0.0.1',
                'Android 0.0.1 初始版本',
                '{"feature":["补齐账号登录、我的页面与资料入口，支持基础账号和户外资料管理。","上线装备库与装备图鉴，方便记录个人装备并查看公共装备信息。","支持户外技能与绳结内容浏览，常用内容可离线缓存。","支持行程规划、轨迹导入、轨迹库和地图预览，把出发前资料整理到手机端。","关于页与版本信息改为读取数据库，便于后续按 Android 端独立维护。"],"bug_fix":[],"notes":[]}',
                NULL,
                'published',
                '2026-06-15T00:00:00Z',
                '2026-06-15T00:00:00Z',
                '2026-06-15T00:00:00Z'
            ) ON CONFLICT(client_key, version) DO UPDATE SET
                title = excluded.title,
                release_notes_json = excluded.release_notes_json,
                commit_hash = excluded.commit_hash,
                status = excluded.status,
                published_at = excluded.published_at,
                updated_at = excluded.updated_at"#,
        )
        .await?;
        Ok(())
    }

    /// Runs schema rollback logic and tries to undo tables or indexes created by up.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS client_versions")
            .await?;
        Ok(())
    }
}
