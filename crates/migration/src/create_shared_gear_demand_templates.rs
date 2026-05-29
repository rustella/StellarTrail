//! Creates DB-backed common shared-gear demand templates for trips.

use sea_orm_migration::prelude::*;

/// Migration adding backend-owned shared-gear demand templates.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "create_shared_gear_demand_templates"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Creates the template table and seeds the first common outdoor shared-gear demands.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE TABLE IF NOT EXISTS shared_gear_demand_templates (
                template_key TEXT PRIMARY KEY,
                demand_name TEXT NOT NULL,
                group_label TEXT NOT NULL,
                category TEXT NOT NULL,
                category_label TEXT NOT NULL,
                planned_quantity INTEGER NOT NULL DEFAULT 1,
                source TEXT NOT NULL DEFAULT 'system_seed',
                status TEXT NOT NULL DEFAULT 'active',
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )"#,
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_shared_gear_demand_templates_status_order \
             ON shared_gear_demand_templates(status, sort_order, template_key)",
        )
        .await?;
        db.execute_unprepared(
            r#"INSERT INTO shared_gear_demand_templates
                (template_key, demand_name, group_label, category, category_label, planned_quantity, source, status, sort_order, created_at, updated_at)
            VALUES
                ('common_stove_burner', '炉头', '炉具', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 10, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_cook_pot', '煮锅', '炉具', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 20, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_frying_pan', '煎锅', '炉具', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 30, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_kettle', '水壶', '炉具', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 40, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_gas_canister', '气罐', '炉具', 'consumable', '消耗品', 1, 'system_seed', 'active', 50, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_windscreen', '挡风板', '炉具', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 60, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_ladle_trowel', '汤勺、小铲', '炉具', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 70, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_camp_knife', '切菜小刀', '炉具', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 80, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_water_bag', '提水袋', '水具', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 90, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_water_filter', '滤水器', '水具', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 100, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_lighter', '点火器', '生活', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 110, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_tray', '菜盘', '生活', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 120, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_oil_salt_container', '便携油盐罐', '生活', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 130, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_cutting_board', '菜板', '生活', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 140, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_picnic_mat', '野外餐垫', '生活', 'kitchen_system', '餐厨系统', 1, 'system_seed', 'active', 150, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_tarp', '天幕', '生活', 'other_gear', '其它装备', 1, 'system_seed', 'active', 160, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_trash_bag', '垃圾袋', '生活', 'consumable', '消耗品', 1, 'system_seed', 'active', 170, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_paper_towels_roll', '纸巾（卷）', '生活', 'consumable', '消耗品', 1, 'system_seed', 'active', 180, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_entrenching_tool', '工兵铲', '生活', 'other_gear', '其它装备', 1, 'system_seed', 'active', 190, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_emergency_whistle', '急救口哨', '应急', 'first_aid_system', '急救系统', 1, 'system_seed', 'active', 200, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_first_aid_kit', '急救包', '应急', 'first_aid_system', '急救系统', 1, 'system_seed', 'active', 210, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_lantern', '营灯', '灯具', 'lighting_system', '照明系统', 1, 'system_seed', 'active', 220, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_walkie_talkie', '对讲机', '通讯', 'electronics_system', '电子系统', 1, 'system_seed', 'active', 230, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_compass', '指北针', '安全', 'technical_gear', '技术装备', 1, 'system_seed', 'active', 240, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_safety_rope', '安全绳', '安全', 'technical_gear', '技术装备', 1, 'system_seed', 'active', 250, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_harness', '安全带', '安全', 'technical_gear', '技术装备', 1, 'system_seed', 'active', 260, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_main_carabiner', '主锁', '安全', 'technical_gear', '技术装备', 1, 'system_seed', 'active', 270, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('common_aux_rope', '辅绳', '安全', 'technical_gear', '技术装备', 1, 'system_seed', 'active', 280, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            ON CONFLICT(template_key) DO UPDATE SET
                demand_name = excluded.demand_name,
                group_label = excluded.group_label,
                category = excluded.category,
                category_label = excluded.category_label,
                planned_quantity = excluded.planned_quantity,
                source = excluded.source,
                status = excluded.status,
                sort_order = excluded.sort_order,
                updated_at = excluded.updated_at"#,
        )
        .await?;
        Ok(())
    }

    /// Drops shared-gear template data introduced by this migration.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS idx_shared_gear_demand_templates_status_order")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS shared_gear_demand_templates")
            .await?;
        Ok(())
    }
}
