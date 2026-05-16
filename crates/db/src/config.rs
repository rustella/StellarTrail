//! Database configuration module that detects the database type from the connection string and provides SeaORM connection settings.

use thiserror::Error;

/// Stable enum boundary for `DatabaseKind`, exposed by or reused within this module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatabaseKind {
    Sqlite,
    Postgres,
    MySql,
}

impl DatabaseKind {
    /// Runs the `as str` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
            Self::MySql => "mysql",
        }
    }
}

/// Stable data boundary for `DatabaseConfig`, exposed by or reused within this module.
#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub kind: DatabaseKind,
}

impl DatabaseConfig {
    /// Runs the `new` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn new(url: String) -> Result<Self, DatabaseConfigError> {
        let kind = if url.starts_with("sqlite:") {
            DatabaseKind::Sqlite
        } else if url.starts_with("postgres:") || url.starts_with("postgresql:") {
            DatabaseKind::Postgres
        } else if url.starts_with("mysql:") {
            DatabaseKind::MySql
        } else {
            return Err(DatabaseConfigError::UnsupportedUrl(url));
        };

        Ok(Self { url, kind })
    }
}

/// Stable enum boundary for `DatabaseConfigError`, exposed by or reused within this module.
#[derive(Debug, Error)]
pub enum DatabaseConfigError {
    #[error("unsupported database url: {0}")]
    UnsupportedUrl(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs the `detects supported database kinds` server-side flow while preserving input validation, error propagation, and state invariants.
    #[test]
    fn detects_supported_database_kinds() {
        let cases = [
            ("sqlite://stellartrail.db", DatabaseKind::Sqlite),
            ("postgres://user:pass@localhost/db", DatabaseKind::Postgres),
            (
                "postgresql://user:pass@localhost/db",
                DatabaseKind::Postgres,
            ),
            ("mysql://user:pass@localhost/db", DatabaseKind::MySql),
        ];

        for (url, expected) in cases {
            let config = DatabaseConfig::new(url.to_owned()).expect("valid database url");
            assert_eq!(config.kind, expected);
        }
    }
}
