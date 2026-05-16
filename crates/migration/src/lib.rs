pub fn migration_plan() -> &'static str {
    "Knots schema is applied by stellartrail-db::KnotRepository::migrate for the MVP SQLite backend."
}

pub fn knots_schema_sql() -> &'static str {
    stellartrail_db::KNOTS_SCHEMA_SQL
}
