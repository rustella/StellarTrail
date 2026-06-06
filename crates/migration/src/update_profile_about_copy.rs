//! Data repair migration for previously seeded WeChat profile About copy.
//!
//! The original seed used `ON CONFLICT DO NOTHING`, so deployed databases that
//! already inserted the old "free forever/no ads" copy do not receive later seed
//! edits. This migration repairs only rows that still contain known legacy
//! markers and leaves unrelated manual database edits untouched.

use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, DatabaseBackend, Statement},
};

/// Updates the default WeChat profile About modal copy when old seeded wording is present.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260607_000003_update_profile_about_copy"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Replaces legacy seeded profile About content without overwriting manual copy edits.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        update_profile_about_copy(db, db.get_database_backend()).await
    }

    /// This data repair is intentionally one-way; rollback should not restore outdated copy.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

async fn update_profile_about_copy(
    db: &SchemaManagerConnection<'_>,
    backend: DatabaseBackend,
) -> Result<(), DbErr> {
    let sql = match backend {
        DatabaseBackend::Postgres => {
            "UPDATE app_content_pages \
             SET content_json = $1, updated_at = $2 \
             WHERE page_key = 'profile_about' \
               AND client_key = 'wechat_miniprogram' \
               AND locale = 'zh-CN' \
               AND (content_json LIKE $3 OR content_json LIKE $4)"
        }
        _ => {
            "UPDATE app_content_pages \
             SET content_json = ?, updated_at = ? \
             WHERE page_key = 'profile_about' \
               AND client_key = 'wechat_miniprogram' \
               AND locale = 'zh-CN' \
               AND (content_json LIKE ? OR content_json LIKE ?)"
        }
    };

    db.execute(Statement::from_sql_and_values(
        backend,
        sql,
        vec![
            CURRENT_PROFILE_ABOUT_CONTENT_JSON.into(),
            PROFILE_ABOUT_UPDATED_AT.into(),
            LEGACY_NO_ADS_MARKER.into(),
            LEGACY_COMMERCIAL_SUPPORT_MARKER.into(),
        ],
    ))
    .await?;

    Ok(())
}

const PROFILE_ABOUT_UPDATED_AT: &str = "2026-06-07T00:00:00Z";
const LEGACY_NO_ADS_MARKER: &str = "%寻径星野会永久免费%";
const LEGACY_COMMERCIAL_SUPPORT_MARKER: &str = "%后续如果引入广告或商业化支持%";
const CURRENT_PROFILE_ABOUT_CONTENT_JSON: &str = r#"{"eyebrow":"🏕️ 寻径星野","title":"关于寻径星野","subtitle":"把每次出发前的准备，整理得更安心。","sections":[{"icon":"🧭","title":"出发准备","body":"寻径星野是一个面向户外爱好者的个人工具，希望把出发前准备、装备管理、装备图鉴、户外技能复习和离线可用的知识内容慢慢整理到一起。"},{"icon":"🎒","title":"山野陪伴","body":"它不只服务某一次路线或某一类装备，而是想陪伴每一次走向山野之前的准备过程：少一点遗漏，多一点安心。"},{"icon":"✨","title":"作者的话","body":"这个项目由作者在业余时间出于爱好开发，也会按自己的使用感受持续打磨。希望它能陪你把每次出发前的准备做得更清楚、更安心。"}],"button_text":"知道了"}"#;

#[cfg(test)]
mod tests {
    use super::{CURRENT_PROFILE_ABOUT_CONTENT_JSON, Migration};
    use sea_orm_migration::{
        prelude::*,
        sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement},
    };

    #[tokio::test]
    async fn rewrites_legacy_no_ads_seed_copy() {
        let db = setup_content_page_db().await;
        insert_profile_about_content(&db, LEGACY_NO_ADS_PROFILE_ABOUT_CONTENT_JSON).await;

        let manager = SchemaManager::new(&db);
        Migration.up(&manager).await.expect("repair legacy copy");

        assert_eq!(
            read_profile_about_content(&db).await,
            CURRENT_PROFILE_ABOUT_CONTENT_JSON
        );
    }

    #[tokio::test]
    async fn rewrites_legacy_commercial_support_seed_copy() {
        let db = setup_content_page_db().await;
        insert_profile_about_content(&db, LEGACY_COMMERCIAL_SUPPORT_PROFILE_ABOUT_CONTENT_JSON)
            .await;

        let manager = SchemaManager::new(&db);
        Migration
            .up(&manager)
            .await
            .expect("repair intermediate legacy copy");

        assert_eq!(
            read_profile_about_content(&db).await,
            CURRENT_PROFILE_ABOUT_CONTENT_JSON
        );
    }

    #[tokio::test]
    async fn keeps_manual_profile_about_copy_unchanged() {
        let db = setup_content_page_db().await;
        let manual_content = r#"{"title":"手动维护的内容","sections":[]}"#;
        insert_profile_about_content(&db, manual_content).await;

        let manager = SchemaManager::new(&db);
        Migration.up(&manager).await.expect("skip manual copy");

        assert_eq!(read_profile_about_content(&db).await, manual_content);
    }

    async fn setup_content_page_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.expect("connect");
        db.execute_unprepared(
            r#"CREATE TABLE app_content_pages (
                id TEXT PRIMARY KEY,
                page_key TEXT NOT NULL,
                client_key TEXT NOT NULL,
                locale TEXT NOT NULL,
                content_json TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(page_key, client_key, locale)
            )"#,
        )
        .await
        .expect("create table");
        db
    }

    async fn insert_profile_about_content(db: &DatabaseConnection, content_json: &str) {
        db.execute(Statement::from_sql_and_values(
            db.get_database_backend(),
            "INSERT INTO app_content_pages (
                id, page_key, client_key, locale, content_json, status, created_at, updated_at
             ) VALUES (?, 'profile_about', 'wechat_miniprogram', 'zh-CN', ?, 'published', ?, ?)",
            vec![
                "profile-about-wechat-zh-cn".into(),
                content_json.into(),
                "2026-06-07T00:00:00Z".into(),
                "2026-06-07T00:00:00Z".into(),
            ],
        ))
        .await
        .expect("insert content");
    }

    async fn read_profile_about_content(db: &DatabaseConnection) -> String {
        let row = db
            .query_one(Statement::from_string(
                db.get_database_backend(),
                "SELECT content_json FROM app_content_pages \
                 WHERE page_key = 'profile_about' \
                   AND client_key = 'wechat_miniprogram' \
                   AND locale = 'zh-CN'"
                    .to_owned(),
            ))
            .await
            .expect("select content")
            .expect("content row");
        row.try_get("", "content_json").expect("content_json")
    }

    const LEGACY_NO_ADS_PROFILE_ABOUT_CONTENT_JSON: &str = r#"{"eyebrow":"🏕️ 寻径星野","title":"关于寻径星野","subtitle":"把每次出发前的准备，整理得更安心。","sections":[{"icon":"🧭","title":"出发准备","body":"寻径星野是一个面向户外爱好者的个人工具，希望把出发前准备、装备管理、装备图鉴、户外技能复习和离线可用的知识内容慢慢整理到一起。"},{"icon":"🎒","title":"山野陪伴","body":"它不只服务某一次路线或某一类装备，而是想陪伴每一次走向山野之前的准备过程：少一点遗漏，多一点安心。"},{"icon":"✨","title":"作者的话","body":"这个项目由作者在业余时间出于爱好开发，也会按自己的使用感受持续打磨。寻径星野会永久免费，无广告，不做打扰用户的商业化设计。"}],"button_text":"知道了"}"#;
    const LEGACY_COMMERCIAL_SUPPORT_PROFILE_ABOUT_CONTENT_JSON: &str = r#"{"eyebrow":"🏕️ 寻径星野","title":"关于寻径星野","subtitle":"把每次出发前的准备，整理得更安心。","sections":[{"icon":"🧭","title":"出发准备","body":"寻径星野是一个面向户外爱好者的个人工具，希望把出发前准备、装备管理、装备图鉴、户外技能复习和离线可用的知识内容慢慢整理到一起。"},{"icon":"🎒","title":"山野陪伴","body":"它不只服务某一次路线或某一类装备，而是想陪伴每一次走向山野之前的准备过程：少一点遗漏，多一点安心。"},{"icon":"✨","title":"作者的话","body":"这个项目由作者在业余时间出于爱好开发，也会按自己的使用感受持续打磨。后续如果引入广告或商业化支持，也会尽量保持克制，优先保证核心功能和出发前准备体验。"}],"button_text":"知道了"}"#;
}
