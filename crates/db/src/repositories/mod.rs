pub mod auth_repository;
pub mod gear_repository;

pub use auth_repository::{AuthRepository, UserRecord, hash_token};
pub use gear_repository::{GearRepository, ListGearOptions};

use sea_orm::{DatabaseBackend, Statement, Value};

pub(crate) fn statement(
    backend: DatabaseBackend,
    sql: impl Into<String>,
    values: Vec<Value>,
) -> Statement {
    let sql = sql.into();
    let sql = if matches!(backend, DatabaseBackend::Postgres) {
        postgres_placeholders(&sql)
    } else {
        sql
    };
    Statement::from_sql_and_values(backend, sql, values)
}

fn postgres_placeholders(sql: &str) -> String {
    let mut converted = String::with_capacity(sql.len());
    let mut index = 1;
    let mut in_single_quote = false;
    let mut chars = sql.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\'' {
            converted.push(ch);
            if in_single_quote && chars.peek() == Some(&'\'') {
                converted.push(chars.next().expect("peeked escaped quote"));
            } else {
                in_single_quote = !in_single_quote;
            }
            continue;
        }

        if ch == '?' && !in_single_quote {
            converted.push('$');
            converted.push_str(&index.to_string());
            index += 1;
        } else {
            converted.push(ch);
        }
    }

    converted
}

#[cfg(test)]
mod tests {
    use super::postgres_placeholders;

    #[test]
    fn converts_question_mark_placeholders_for_postgres() {
        assert_eq!(
            postgres_placeholders("SELECT * FROM users WHERE id = ? AND email = ?"),
            "SELECT * FROM users WHERE id = $1 AND email = $2",
        );
    }

    #[test]
    fn leaves_question_marks_inside_sql_strings_unchanged() {
        assert_eq!(
            postgres_placeholders(
                "SELECT '?' AS literal, name FROM users WHERE id = ? AND note = 'it''s ?'"
            ),
            "SELECT '?' AS literal, name FROM users WHERE id = $1 AND note = 'it''s ?'",
        );
    }
}
