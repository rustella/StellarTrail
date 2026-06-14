//! Ensures Android profile About copy and initial client-version data.
//!
//! Earlier content-page seeds only created the WeChat Mini Program row. Android
//! now reads the same public content-page table with its own client key, so this
//! data repair inserts the missing Android row while preserving manual edits.
//! Android client-version rows are product-maintained copy, so this migration
//! intentionally keeps only Android 0.0.1 and overwrites that release content.

use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, DatabaseBackend, Statement, Value},
};

/// Inserts the Android profile About content row when it is missing.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260615_000001_ensure_android_profile_about_copy"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Inserts Android DB-backed About copy and cleans known legacy seeded text.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();
        update_legacy_android_profile_about_copy(db, backend).await?;
        insert_android_profile_about_copy(db, backend).await?;
        prune_android_client_versions(db).await?;
        upsert_android_initial_version(db, backend).await?;
        Ok(())
    }

    /// This data repair is intentionally one-way; rollback should not remove content.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

async fn prune_android_client_versions(db: &SchemaManagerConnection<'_>) -> Result<(), DbErr> {
    db.execute_unprepared(
        "DELETE FROM client_versions WHERE client_key = 'android' AND version <> '0.0.1'",
    )
    .await?;
    Ok(())
}

async fn upsert_android_initial_version(
    db: &SchemaManagerConnection<'_>,
    backend: DatabaseBackend,
) -> Result<(), DbErr> {
    let sql = match backend {
        DatabaseBackend::Postgres => {
            "INSERT INTO client_versions (
                id, client_key, version, title, release_notes_json, commit_hash, status,
                published_at, created_by_user_id, updated_by_user_id, created_at, updated_at
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             ON CONFLICT(client_key, version) DO UPDATE SET
                title = EXCLUDED.title,
                release_notes_json = EXCLUDED.release_notes_json,
                commit_hash = EXCLUDED.commit_hash,
                status = EXCLUDED.status,
                published_at = EXCLUDED.published_at,
                updated_by_user_id = EXCLUDED.updated_by_user_id,
                updated_at = EXCLUDED.updated_at"
        }
        _ => {
            "INSERT INTO client_versions (
                id, client_key, version, title, release_notes_json, commit_hash, status,
                published_at, created_by_user_id, updated_by_user_id, created_at, updated_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(client_key, version) DO UPDATE SET
                title = excluded.title,
                release_notes_json = excluded.release_notes_json,
                commit_hash = excluded.commit_hash,
                status = excluded.status,
                published_at = excluded.published_at,
                updated_by_user_id = excluded.updated_by_user_id,
                updated_at = excluded.updated_at"
        }
    };

    db.execute(Statement::from_sql_and_values(
        backend,
        sql,
        vec![
            ANDROID_VERSION_ID.into(),
            "android".into(),
            ANDROID_VERSION.into(),
            ANDROID_VERSION_TITLE.into(),
            ANDROID_VERSION_RELEASE_NOTES_JSON.into(),
            Value::String(None),
            "published".into(),
            ANDROID_VERSION_TIMESTAMP.into(),
            Value::String(None),
            Value::String(None),
            ANDROID_VERSION_TIMESTAMP.into(),
            ANDROID_VERSION_TIMESTAMP.into(),
        ],
    ))
    .await?;
    Ok(())
}

async fn update_legacy_android_profile_about_copy(
    db: &SchemaManagerConnection<'_>,
    backend: DatabaseBackend,
) -> Result<(), DbErr> {
    let sql = match backend {
        DatabaseBackend::Postgres => {
            "UPDATE app_content_pages \
             SET content_json = $1, updated_at = $2 \
             WHERE page_key = 'profile_about' \
               AND client_key = 'android' \
               AND locale = 'zh-CN' \
               AND (content_json LIKE $3 OR content_json LIKE $4)"
        }
        _ => {
            "UPDATE app_content_pages \
             SET content_json = ?, updated_at = ? \
             WHERE page_key = 'profile_about' \
               AND client_key = 'android' \
               AND locale = 'zh-CN' \
               AND (content_json LIKE ? OR content_json LIKE ?)"
        }
    };

    db.execute(Statement::from_sql_and_values(
        backend,
        sql,
        vec![
            PROFILE_ABOUT_CONTENT_JSON.into(),
            PROFILE_ABOUT_UPDATED_AT.into(),
            LEGACY_NO_ADS_MARKER.into(),
            LEGACY_COMMERCIAL_SUPPORT_MARKER.into(),
        ],
    ))
    .await?;
    Ok(())
}

async fn insert_android_profile_about_copy(
    db: &SchemaManagerConnection<'_>,
    backend: DatabaseBackend,
) -> Result<(), DbErr> {
    let sql = match backend {
        DatabaseBackend::Postgres => {
            "INSERT INTO app_content_pages (
                id, page_key, client_key, locale, content_json, status, created_at, updated_at
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT(page_key, client_key, locale) DO NOTHING"
        }
        _ => {
            "INSERT INTO app_content_pages (
                id, page_key, client_key, locale, content_json, status, created_at, updated_at
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(page_key, client_key, locale) DO NOTHING"
        }
    };

    db.execute(Statement::from_sql_and_values(
        backend,
        sql,
        vec![
            "profile-about-android-zh-cn".into(),
            "profile_about".into(),
            "android".into(),
            "zh-CN".into(),
            PROFILE_ABOUT_CONTENT_JSON.into(),
            "published".into(),
            PROFILE_ABOUT_UPDATED_AT.into(),
            PROFILE_ABOUT_UPDATED_AT.into(),
        ],
    ))
    .await?;
    Ok(())
}

const PROFILE_ABOUT_UPDATED_AT: &str = "2026-06-15T00:00:00Z";
const LEGACY_NO_ADS_MARKER: &str = "%寻径星野会永久免费%";
const LEGACY_COMMERCIAL_SUPPORT_MARKER: &str = "%后续如果引入广告或商业化支持%";
const PROFILE_ABOUT_CONTENT_JSON: &str = r#"{"eyebrow":"🏕️ 寻径星野","title":"关于寻径星野","subtitle":"把每次出发前的准备，整理得更安心。","sections":[{"icon":"🧭","title":"出发准备","body":"寻径星野是一个面向户外爱好者的个人工具，希望把出发前准备、装备管理、装备图鉴、户外技能复习和离线可用的知识内容慢慢整理到一起。"},{"icon":"🎒","title":"山野陪伴","body":"它不只服务某一次路线或某一类装备，而是想陪伴每一次走向山野之前的准备过程：少一点遗漏，多一点安心。"},{"icon":"✨","title":"作者的话","body":"这个项目由作者在业余时间出于爱好开发，也会按自己的使用感受持续打磨。希望它能陪你把每次出发前的准备做得更清楚、更安心。"}],"button_text":"知道了"}"#;
const ANDROID_VERSION_ID: &str = "android-0-0-1";
const ANDROID_VERSION: &str = "0.0.1";
const ANDROID_VERSION_TITLE: &str = "Android 0.0.1 初始版本";
const ANDROID_VERSION_TIMESTAMP: &str = "2026-06-15T00:00:00Z";
const ANDROID_VERSION_RELEASE_NOTES_JSON: &str = r#"{"feature":["补齐账号登录、我的页面与资料入口，支持基础账号和户外资料管理。","上线装备库与装备图鉴，方便记录个人装备并查看公共装备信息。","支持户外技能与绳结内容浏览，常用内容可离线缓存。","支持行程规划、轨迹导入、轨迹库和地图预览，把出发前资料整理到手机端。","关于页与版本信息改为读取数据库，便于后续按 Android 端独立维护。"],"bug_fix":[],"notes":[]}"#;

#[cfg(test)]
mod tests {
    use super::{
        ANDROID_VERSION_RELEASE_NOTES_JSON, ANDROID_VERSION_TIMESTAMP, Migration,
        PROFILE_ABOUT_CONTENT_JSON,
    };
    use sea_orm_migration::{
        prelude::*,
        sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement},
    };

    #[tokio::test]
    async fn inserts_missing_android_profile_about_copy() {
        let db = setup_content_page_db().await;

        let manager = SchemaManager::new(&db);
        Migration.up(&manager).await.expect("insert android copy");

        assert_eq!(
            read_profile_about_content(&db, "android").await,
            PROFILE_ABOUT_CONTENT_JSON
        );
    }

    #[tokio::test]
    async fn keeps_manual_android_profile_about_copy_unchanged() {
        let db = setup_content_page_db().await;
        let manual_content = r#"{"eyebrow":"维护","title":"手动维护","subtitle":"手动内容","sections":[{"icon":"手","title":"手动","body":"不要覆盖"}],"button_text":"知道了"}"#;
        insert_profile_about_content(&db, "android", manual_content).await;

        let manager = SchemaManager::new(&db);
        Migration.up(&manager).await.expect("preserve manual copy");

        assert_eq!(
            read_profile_about_content(&db, "android").await,
            manual_content
        );
    }

    #[tokio::test]
    async fn rewrites_legacy_android_seed_copy() {
        let db = setup_content_page_db().await;
        insert_profile_about_content(&db, "android", LEGACY_NO_ADS_PROFILE_ABOUT_CONTENT_JSON)
            .await;

        let manager = SchemaManager::new(&db);
        Migration.up(&manager).await.expect("repair legacy copy");

        assert_eq!(
            read_profile_about_content(&db, "android").await,
            PROFILE_ABOUT_CONTENT_JSON
        );
    }

    #[tokio::test]
    async fn inserts_android_client_version_zero_zero_one_when_missing() {
        let db = setup_content_page_db().await;

        let manager = SchemaManager::new(&db);
        Migration
            .up(&manager)
            .await
            .expect("insert android version");

        let record = read_android_zero_zero_one(&db).await;
        assert_eq!(record.title, "Android 0.0.1 初始版本");
        assert_eq!(
            record.release_notes_json,
            ANDROID_VERSION_RELEASE_NOTES_JSON
        );
        assert_eq!(record.status, "published");
        assert_eq!(
            record.published_at.as_deref(),
            Some(ANDROID_VERSION_TIMESTAMP)
        );
    }

    #[tokio::test]
    async fn overwrites_existing_android_client_version_zero_zero_one_content() {
        let db = setup_content_page_db().await;
        insert_client_version(
            &db,
            "manual-android-0-0-1",
            "android",
            "0.0.1",
            "人工维护旧内容",
            r#"{"feature":["旧内容"]}"#,
            "draft",
        )
        .await;

        let manager = SchemaManager::new(&db);
        Migration
            .up(&manager)
            .await
            .expect("overwrite android version");

        let record = read_android_zero_zero_one(&db).await;
        assert_eq!(record.title, "Android 0.0.1 初始版本");
        assert_eq!(
            record.release_notes_json,
            ANDROID_VERSION_RELEASE_NOTES_JSON
        );
        assert_eq!(record.status, "published");
    }

    #[tokio::test]
    async fn removes_other_android_client_versions_but_keeps_other_clients() {
        let db = setup_content_page_db().await;
        insert_client_version(
            &db,
            "android-0-1-0",
            "android",
            "0.1.0",
            "Android 0.1.0",
            r#"{"feature":["不再保留"]}"#,
            "published",
        )
        .await;
        insert_client_version(
            &db,
            "ios-0-1-0",
            "ios",
            "0.1.0",
            "iOS 0.1.0",
            r#"{"feature":["保留"]}"#,
            "published",
        )
        .await;

        let manager = SchemaManager::new(&db);
        Migration
            .up(&manager)
            .await
            .expect("prune android versions");

        assert_eq!(read_versions(&db, "android").await, vec!["0.0.1"]);
        assert_eq!(read_versions(&db, "ios").await, vec!["0.1.0"]);
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
        db.execute_unprepared(
            r#"CREATE TABLE client_versions (
                id TEXT PRIMARY KEY,
                client_key TEXT NOT NULL,
                version TEXT NOT NULL,
                title TEXT NOT NULL,
                release_notes_json TEXT NOT NULL DEFAULT '[]',
                commit_hash TEXT NULL,
                status TEXT NOT NULL,
                published_at TEXT NULL,
                created_by_user_id TEXT NULL,
                updated_by_user_id TEXT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(client_key, version)
            )"#,
        )
        .await
        .expect("create client versions table");
        db
    }

    async fn insert_profile_about_content(
        db: &DatabaseConnection,
        client_key: &str,
        content_json: &str,
    ) {
        db.execute(Statement::from_sql_and_values(
            db.get_database_backend(),
            "INSERT INTO app_content_pages (
                id, page_key, client_key, locale, content_json, status, created_at, updated_at
             ) VALUES (?, 'profile_about', ?, 'zh-CN', ?, 'published', ?, ?)",
            vec![
                format!("profile-about-{client_key}-zh-cn").into(),
                client_key.to_owned().into(),
                content_json.into(),
                "2026-06-15T00:00:00Z".into(),
                "2026-06-15T00:00:00Z".into(),
            ],
        ))
        .await
        .expect("insert content");
    }

    async fn read_profile_about_content(db: &DatabaseConnection, client_key: &str) -> String {
        let row = db
            .query_one(Statement::from_sql_and_values(
                db.get_database_backend(),
                "SELECT content_json FROM app_content_pages \
                 WHERE page_key = 'profile_about' \
                   AND client_key = ? \
                   AND locale = 'zh-CN'",
                vec![client_key.to_owned().into()],
            ))
            .await
            .expect("select content")
            .expect("content row");
        row.try_get("", "content_json").expect("content_json")
    }

    async fn insert_client_version(
        db: &DatabaseConnection,
        id: &str,
        client_key: &str,
        version: &str,
        title: &str,
        release_notes_json: &str,
        status: &str,
    ) {
        db.execute(Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"INSERT INTO client_versions (
                id, client_key, version, title, release_notes_json, commit_hash, status,
                published_at, created_by_user_id, updated_by_user_id, created_at, updated_at
             ) VALUES (?, ?, ?, ?, ?, NULL, ?, ?, NULL, NULL, ?, ?)"#,
            vec![
                id.to_owned().into(),
                client_key.to_owned().into(),
                version.to_owned().into(),
                title.to_owned().into(),
                release_notes_json.to_owned().into(),
                status.to_owned().into(),
                ANDROID_VERSION_TIMESTAMP.into(),
                ANDROID_VERSION_TIMESTAMP.into(),
                ANDROID_VERSION_TIMESTAMP.into(),
            ],
        ))
        .await
        .expect("insert client version");
    }

    async fn read_android_zero_zero_one(db: &DatabaseConnection) -> ClientVersionRow {
        let row = db
            .query_one(Statement::from_sql_and_values(
                db.get_database_backend(),
                "SELECT title, release_notes_json, status, published_at FROM client_versions \
                 WHERE client_key = 'android' AND version = '0.0.1'",
                vec![],
            ))
            .await
            .expect("select android 0.0.1")
            .expect("android 0.0.1 row");
        ClientVersionRow {
            title: row.try_get("", "title").expect("title"),
            release_notes_json: row
                .try_get("", "release_notes_json")
                .expect("release_notes_json"),
            status: row.try_get("", "status").expect("status"),
            published_at: row.try_get("", "published_at").expect("published_at"),
        }
    }

    async fn read_versions(db: &DatabaseConnection, client_key: &str) -> Vec<String> {
        db.query_all(Statement::from_sql_and_values(
            db.get_database_backend(),
            "SELECT version FROM client_versions WHERE client_key = ? ORDER BY version",
            vec![client_key.to_owned().into()],
        ))
        .await
        .expect("select versions")
        .into_iter()
        .map(|row| row.try_get("", "version").expect("version"))
        .collect()
    }

    struct ClientVersionRow {
        title: String,
        release_notes_json: String,
        status: String,
        published_at: Option<String>,
    }

    const LEGACY_NO_ADS_PROFILE_ABOUT_CONTENT_JSON: &str = r#"{"eyebrow":"🏕️ 寻径星野","title":"关于寻径星野","subtitle":"把每次出发前的准备，整理得更安心。","sections":[{"icon":"🧭","title":"出发准备","body":"寻径星野是一个面向户外爱好者的个人工具，希望把出发前准备、装备管理、装备图鉴、户外技能复习和离线可用的知识内容慢慢整理到一起。"},{"icon":"🎒","title":"山野陪伴","body":"它不只服务某一次路线或某一类装备，而是想陪伴每一次走向山野之前的准备过程：少一点遗漏，多一点安心。"},{"icon":"✨","title":"作者的话","body":"这个项目由作者在业余时间出于爱好开发，也会按自己的使用感受持续打磨。寻径星野会永久免费，无广告，不做打扰用户的商业化设计。"}],"button_text":"知道了"}"#;
}
