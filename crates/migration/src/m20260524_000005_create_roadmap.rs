//! Roadmap migration for DB-backed product planning and account-scoped interest signals.
//!
//! Roadmap rows are deliberately separate from route, skill, and gear feature
//! tables. A roadmap item can describe future work without implying that the
//! described product module exists in the runtime API.

use sea_orm_migration::prelude::*;

/// Single migration type for roadmap content, votes, subscriptions, and seed rows.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates roadmap tables, interaction indexes, and the initial WeChat roadmap items.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS roadmap_items (
                id TEXT PRIMARY KEY,
                client_key TEXT NOT NULL CHECK (client_key IN ('wechat_miniprogram', 'web', 'android', 'ios', 'macos')),
                title TEXT NOT NULL,
                summary TEXT NOT NULL,
                details TEXT NULL,
                category TEXT NOT NULL CHECK (category IN ('gear', 'skills', 'routes', 'offline', 'safety', 'community')),
                status TEXT NOT NULL CHECK (status IN ('planned', 'designing', 'building', 'preview', 'shipped')),
                priority INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                is_published BOOLEAN NOT NULL DEFAULT TRUE,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                published_at TEXT NULL,
                created_by_user_id TEXT NULL REFERENCES users(id) ON DELETE SET NULL,
                updated_by_user_id TEXT NULL REFERENCES users(id) ON DELETE SET NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_roadmap_items_public \
             ON roadmap_items(client_key, is_published, is_deleted, status, sort_order, priority)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_roadmap_items_admin \
             ON roadmap_items(client_key, status, is_deleted, updated_at)",
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS user_roadmap_votes (
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                roadmap_item_id TEXT NOT NULL REFERENCES roadmap_items(id) ON DELETE CASCADE,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                voted_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (user_id, roadmap_item_id)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_roadmap_votes_item_active \
             ON user_roadmap_votes(roadmap_item_id, is_deleted)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_roadmap_votes_user_active \
             ON user_roadmap_votes(user_id, is_deleted, voted_at)",
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS user_roadmap_subscriptions (
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                roadmap_item_id TEXT NOT NULL REFERENCES roadmap_items(id) ON DELETE CASCADE,
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
                subscribed_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (user_id, roadmap_item_id)
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_roadmap_subscriptions_item_active \
             ON user_roadmap_subscriptions(roadmap_item_id, is_deleted)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_user_roadmap_subscriptions_user_active \
             ON user_roadmap_subscriptions(user_id, is_deleted, subscribed_at)",
        )
        .await?;

        for seed in ROADMAP_SEEDS {
            db.execute_unprepared(&seed.insert_sql()).await?;
        }
        Ok(())
    }

    /// Removes roadmap interaction tables and item content.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_user_roadmap_subscriptions_user_active")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_user_roadmap_subscriptions_item_active")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS user_roadmap_subscriptions")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_user_roadmap_votes_user_active")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_user_roadmap_votes_item_active")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS user_roadmap_votes")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_roadmap_items_admin")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_roadmap_items_public")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS roadmap_items")
            .await?;
        Ok(())
    }
}

struct RoadmapSeed {
    id: &'static str,
    title: &'static str,
    summary: &'static str,
    details: &'static str,
    category: &'static str,
    priority: i32,
    sort_order: i32,
}

impl RoadmapSeed {
    fn insert_sql(&self) -> String {
        format!(
            r#"INSERT INTO roadmap_items (
                id, client_key, title, summary, details, category, status,
                priority, sort_order, is_published, is_deleted, published_at,
                created_at, updated_at
            ) VALUES (
                '{}',
                'wechat_miniprogram',
                '{}',
                '{}',
                '{}',
                '{}',
                'planned',
                {},
                {},
                TRUE,
                FALSE,
                '2026-05-24T00:00:00Z',
                '2026-05-24T00:00:00Z',
                '2026-05-24T00:00:00Z'
            ) ON CONFLICT(id) DO NOTHING"#,
            self.id,
            self.title,
            self.summary,
            self.details,
            self.category,
            self.priority,
            self.sort_order,
        )
    }
}

const ROADMAP_SEEDS: &[RoadmapSeed] = &[
    RoadmapSeed {
        id: "smart-packing-template",
        title: "智能打包清单模板",
        summary: "按路线或目的地、天数和季节，结合个人装备和历史打包习惯生成建议清单。",
        details: "第一版会先整理影响打包建议的输入项和清单生成入口，本次 Roadmap 只记录规划，不实现推荐算法。",
        category: "gear",
        priority: 100,
        sort_order: 10,
    },
    RoadmapSeed {
        id: "knot-scenario-videos",
        title: "绳结实际场景视频演示",
        summary: "为常用绳结补充露营、固定、连接和收纳等真实使用场景视频。",
        details: "在现有绳结动图之外，补充更贴近户外使用的短视频演示，帮助用户理解何时使用。",
        category: "skills",
        priority: 90,
        sort_order: 20,
    },
    RoadmapSeed {
        id: "route-encyclopedia",
        title: "路线百科",
        summary: "展示路线难度、季节、风险、交通和准备要点。",
        details: "路线百科仍属于未来模块，本次只作为 Roadmap 条目出现，不新增路线 API 或路线内容表。",
        category: "routes",
        priority: 80,
        sort_order: 30,
    },
    RoadmapSeed {
        id: "skill-scenario-index",
        title: "按场景查找技能",
        summary: "按扎营固定、收纳、连接、应急等场景查找绳结和户外技能。",
        details: "把技能内容从单纯分类扩展到实际问题场景，让出发前复习更直接。",
        category: "skills",
        priority: 70,
        sort_order: 40,
    },
    RoadmapSeed {
        id: "gear-maintenance-reminders",
        title: "装备维护提醒",
        summary: "记录装备保养、充电、耗材补充和有效期提醒。",
        details: "围绕已有个人装备库增加轻量提醒能力，帮助用户在出发前发现需要处理的装备。",
        category: "gear",
        priority: 60,
        sort_order: 50,
    },
    RoadmapSeed {
        id: "offline-trip-pack",
        title: "离线出行包",
        summary: "一键缓存出行前需要的技能、清单、装备和安全资料。",
        details: "延续现有离线只读能力，把零散缓存整理成一次出行可用的离线资料包。",
        category: "offline",
        priority: 50,
        sort_order: 60,
    },
    RoadmapSeed {
        id: "safety-weather-precheck",
        title: "安全与天气预检查",
        summary: "出发前天气、风险和急救检查清单。",
        details: "把天气、风险提示和基础急救准备整理为出发前确认流程。",
        category: "safety",
        priority: 40,
        sort_order: 70,
    },
    RoadmapSeed {
        id: "learning-progress",
        title: "技能学习进度",
        summary: "记录技能学习进度、已掌握标记和复习提醒。",
        details: "帮助用户追踪已经学过或需要复习的绳结与户外技能。",
        category: "skills",
        priority: 30,
        sort_order: 80,
    },
];
