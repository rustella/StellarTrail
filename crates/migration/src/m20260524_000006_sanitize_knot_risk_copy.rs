//! Data repair migration that softens high-risk public knot copy imported from Knots3D.

use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, DatabaseBackend, Statement},
};

/// Rewrites a few public Chinese knot descriptions that read like operational safety guidance.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Applies curated copy updates for existing production rows.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        for fix in RISK_COPY_FIXES {
            update_knot_copy(db, backend, fix).await?;
        }

        Ok(())
    }

    /// This copy repair is intentionally one-way so rollback does not restore unsafe wording.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

async fn update_knot_copy(
    db: &SchemaManagerConnection<'_>,
    backend: DatabaseBackend,
    fix: &KnotCopyFix,
) -> Result<(), DbErr> {
    let sql = match (backend, fix.field) {
        (DatabaseBackend::Postgres, KnotCopyField::Summary) => {
            "UPDATE knot_localizations SET summary = $1 WHERE knot_id = $2 AND locale = $3"
        }
        (DatabaseBackend::Postgres, KnotCopyField::Description) => {
            "UPDATE knot_localizations SET description = $1 WHERE knot_id = $2 AND locale = $3"
        }
        (_, KnotCopyField::Summary) => {
            "UPDATE knot_localizations SET summary = ? WHERE knot_id = ? AND locale = ?"
        }
        (_, KnotCopyField::Description) => {
            "UPDATE knot_localizations SET description = ? WHERE knot_id = ? AND locale = ?"
        }
    };

    db.execute(Statement::from_sql_and_values(
        backend,
        sql,
        vec![fix.value.into(), fix.knot_id.into(), fix.locale.into()],
    ))
    .await?;

    Ok(())
}

#[derive(Clone, Copy)]
enum KnotCopyField {
    Summary,
    Description,
}

struct KnotCopyFix {
    knot_id: &'static str,
    locale: &'static str,
    field: KnotCopyField,
    value: &'static str,
}

const RISK_COPY_FIXES: &[KnotCopyFix] = &[
    KnotCopyFix {
        knot_id: "figure-eight-follow-through-knot",
        locale: "zh-CN",
        field: KnotCopyField::Description,
        value: "八字返穿结常用于需要在绳端形成固定绳环的学习示例，也常见于攀岩教材和绳索课程。它结构清晰，便于观察绳路是否顺直；受力后可能变得难以解开。请仅将本页面作为结形识别和练习参考，任何攀登、探洞或高风险活动都应以专业培训、合规装备和现场指导为准。",
    },
    KnotCopyFix {
        knot_id: "firemans-chair-knot",
        locale: "zh-CN",
        field: KnotCopyField::Description,
        value: "消防员椅结是一种在绳索中段形成两个可调绳环的传统结法，本页面仅用于介绍其结构和历史用途。它涉及救援语境和人员保护，实际场景必须由受训人员使用合规装备与流程完成；不要把这里的示意当作操作指南。",
    },
    KnotCopyFix {
        knot_id: "handcuff-knot",
        locale: "zh-CN",
        field: KnotCopyField::Description,
        value: "手铐结系在绳子中间，形成两个可调绳环，名称来自外形特征。该结本身不具备可靠锁定能力，本页面仅适合了解结形和结构；涉及人员保护、救助或约束的实际场景，应使用专业设备和规范流程。",
    },
    KnotCopyFix {
        knot_id: "shear-lashing-knot",
        locale: "zh-CN",
        field: KnotCopyField::Summary,
        value: "将两根杆件交叉绑扎，适合学习支架和营地结构的绑法。",
    },
    KnotCopyFix {
        knot_id: "spanish-bowline-knot",
        locale: "zh-CN",
        field: KnotCopyField::Summary,
        value: "形成两个并列固定绳环，适合学习双环结结构。",
    },
];
