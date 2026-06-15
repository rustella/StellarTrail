//! Creates DB-backed application content pages for client-facing copy.
//!
//! These rows are intentionally small, structured content documents that can be
//! changed in deployed databases without shipping a new Mini Program bundle.

use sea_orm_migration::prelude::*;

/// Migration creating application content page storage and initial profile About copy.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260607_000002_create_app_content_pages"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates the content page table and seeds client-specific profile About pages.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS app_content_pages (
                id TEXT PRIMARY KEY,
                page_key TEXT NOT NULL,
                client_key TEXT NOT NULL CHECK (client_key IN ('wechat_miniprogram', 'web', 'android', 'ios', 'macos')),
                locale TEXT NOT NULL,
                content_json TEXT NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('draft', 'published', 'archived')),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(page_key, client_key, locale)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_app_content_pages_public \
             ON app_content_pages(page_key, client_key, locale, status)",
        )
        .await?;
        db.execute_unprepared(
            r#"INSERT INTO app_content_pages (
                id, page_key, client_key, locale, content_json, status, created_at, updated_at
            ) VALUES (
                'profile-about-wechat-zh-cn',
                'profile_about',
                'wechat_miniprogram',
                'zh-CN',
                '{"eyebrow":"🏕️ 寻径星野","title":"关于寻径星野","subtitle":"把每次出发前的准备，整理得更安心。","sections":[{"icon":"🧭","title":"出发准备","body":"寻径星野是一个面向户外爱好者的个人工具，希望把出发前准备、装备管理、装备图鉴、户外技能复习和离线可用的知识内容慢慢整理到一起。"},{"icon":"🎒","title":"山野陪伴","body":"它不只服务某一次路线或某一类装备，而是想陪伴每一次走向山野之前的准备过程：少一点遗漏，多一点安心。"},{"icon":"✨","title":"作者的话","body":"这个项目由作者在业余时间出于爱好开发，也会按自己的使用感受持续打磨。希望它能陪你把每次出发前的准备做得更清楚、更安心。"}],"button_text":"知道了"}',
                'published',
                '2026-06-07T00:00:00Z',
                '2026-06-07T00:00:00Z'
            ) ON CONFLICT(page_key, client_key, locale) DO NOTHING"#,
        )
        .await?;
        db.execute_unprepared(
            r#"INSERT INTO app_content_pages (
                id, page_key, client_key, locale, content_json, status, created_at, updated_at
            ) VALUES (
                'profile-about-android-zh-cn',
                'profile_about',
                'android',
                'zh-CN',
                '{"eyebrow":"🏕️ 寻径星野","title":"关于寻径星野","subtitle":"把每次出发前的准备，整理得更安心。","sections":[{"icon":"🧭","title":"出发准备","body":"寻径星野是一个面向户外爱好者的个人工具，希望把出发前准备、装备管理、装备图鉴、户外技能复习和离线可用的知识内容慢慢整理到一起。"},{"icon":"🎒","title":"山野陪伴","body":"它不只服务某一次路线或某一类装备，而是想陪伴每一次走向山野之前的准备过程：少一点遗漏，多一点安心。"},{"icon":"✨","title":"作者的话","body":"这个项目由作者在业余时间出于爱好开发，也会按自己的使用感受持续打磨。希望它能陪你把每次出发前的准备做得更清楚、更安心。"}],"button_text":"知道了"}',
                'published',
                '2026-06-15T00:00:00Z',
                '2026-06-15T00:00:00Z'
            ) ON CONFLICT(page_key, client_key, locale) DO NOTHING"#,
        )
        .await?;
        Ok(())
    }

    /// Drops application content page storage.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_app_content_pages_public")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS app_content_pages")
            .await?;
        Ok(())
    }
}
