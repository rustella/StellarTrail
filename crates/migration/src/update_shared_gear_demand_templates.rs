//! Updates shared-gear templates after field testing the first template set.

use sea_orm_migration::prelude::*;

/// Migration that refines public gear demand slots for trips.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "update_shared_gear_demand_templates"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Moves water/table items into the kitchen system and splits combined safety gear.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"UPDATE shared_gear_demand_templates
               SET category = 'kitchen_system',
                   category_label = '餐厨系统',
                   updated_at = CURRENT_TIMESTAMP
               WHERE template_key IN ('common_water_bag', 'common_water_filter', 'common_picnic_mat')"#,
        )
        .await?;
        db.execute_unprepared(
            r#"UPDATE shared_gear_demand_templates
               SET status = 'inactive',
                   updated_at = CURRENT_TIMESTAMP
               WHERE template_key = 'common_carabiners_aux_rope'"#,
        )
        .await?;
        db.execute_unprepared(
            r#"INSERT INTO shared_gear_demand_templates
                (template_key, demand_name, group_label, category, category_label, planned_quantity, source, status, sort_order, created_at, updated_at)
            VALUES
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

    /// Reverts template refinements for migration rollback.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"UPDATE shared_gear_demand_templates
               SET category = 'other_gear',
                   category_label = '其它装备',
                   updated_at = CURRENT_TIMESTAMP
               WHERE template_key IN ('common_water_bag', 'common_water_filter', 'common_picnic_mat')"#,
        )
        .await?;
        db.execute_unprepared(
            "DELETE FROM shared_gear_demand_templates \
             WHERE template_key IN ('common_main_carabiner', 'common_aux_rope')",
        )
        .await?;
        db.execute_unprepared(
            r#"UPDATE shared_gear_demand_templates
               SET status = 'active',
                   sort_order = 270,
                   updated_at = CURRENT_TIMESTAMP
               WHERE template_key = 'common_carabiners_aux_rope'"#,
        )
        .await?;
        Ok(())
    }
}
