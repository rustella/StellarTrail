//! Data repair migration for Knots3D rows whose narrative section headings were imported as practice steps.

use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, DatabaseBackend, Statement},
};

/// Clears bogus Knots3D practice steps that were populated from page section headings.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Removes already-imported section headings from Knots3D localization steps.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();
        let rows = db
            .query_all(Statement::from_string(
                backend,
                "SELECT kl.knot_id, kl.locale, kl.steps_json \
                 FROM knot_localizations kl \
                 JOIN knots k ON k.id = kl.knot_id \
                 WHERE k.source_name = 'Knots 3D' AND kl.steps_json <> '[]'"
                    .to_owned(),
            ))
            .await?;

        let update_sql = match backend {
            DatabaseBackend::Postgres => {
                "UPDATE knot_localizations SET steps_json = '[]' WHERE knot_id = $1 AND locale = $2"
            }
            _ => "UPDATE knot_localizations SET steps_json = '[]' WHERE knot_id = ? AND locale = ?",
        };

        for row in rows {
            let steps_json: String = row.try_get("", "steps_json")?;
            if !looks_like_knots3d_section_headings(&steps_json) {
                continue;
            }

            let knot_id: String = row.try_get("", "knot_id")?;
            let locale: String = row.try_get("", "locale")?;
            db.execute(Statement::from_sql_and_values(
                backend,
                update_sql,
                vec![knot_id.into(), locale.into()],
            ))
            .await?;
        }

        Ok(())
    }

    /// This data repair is intentionally not reversible; restoring the bad headings would reintroduce the bug.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

fn looks_like_knots3d_section_headings(steps_json: &str) -> bool {
    let Ok(steps) = serde_json::from_str::<Vec<String>>(steps_json) else {
        return false;
    };

    !steps.is_empty()
        && steps
            .iter()
            .all(|step| KNOTS3D_SECTION_HEADINGS.contains(&step.as_str()))
}

const KNOTS3D_SECTION_HEADINGS: &[&str] = &[
    "Usage",
    "Warning ⚠️",
    "History",
    "Also known as",
    "Related",
    "ABOK",
    "Structure",
    "Strength/Reliability",
    "Note",
    "用途",
    "警告 ⚠️",
    "也叫作",
    "相关",
];

#[cfg(test)]
mod tests {
    use super::looks_like_knots3d_section_headings;

    #[test]
    fn recognizes_imported_knots3d_section_heading_steps() {
        assert!(looks_like_knots3d_section_headings(
            r#"["用途","警告 ⚠️","也叫作","相关","ABOK"]"#
        ));
        assert!(looks_like_knots3d_section_headings(
            r#"["Usage","Warning ⚠️","History","Also known as","Related","ABOK","Structure","Strength/Reliability"]"#
        ));
    }

    #[test]
    fn keeps_real_steps_and_empty_steps_unchanged() {
        assert!(!looks_like_knots3d_section_headings(r#"[]"#));
        assert!(!looks_like_knots3d_section_headings(
            r#"["绕出一个绳圈。","将绳头穿过绳圈。"]"#
        ));
        assert!(!looks_like_knots3d_section_headings(
            r#"["Usage","将绳头穿过绳圈。"]"#
        ));
        assert!(!looks_like_knots3d_section_headings("not json"));
    }
}
