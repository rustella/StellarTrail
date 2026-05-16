//! 数据库配置模块，根据连接串识别数据库类型并为 SeaORM 建立连接提供参数。

use thiserror::Error;

/// DatabaseKind 枚举，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatabaseKind {
    Sqlite,
    Postgres,
    MySql,
}

impl DatabaseKind {
    /// 执行 `as str` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
            Self::MySql => "mysql",
        }
    }
}

/// DatabaseConfig 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub kind: DatabaseKind,
}

impl DatabaseConfig {
    /// 执行 `new` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

/// DatabaseConfigError 枚举，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Debug, Error)]
pub enum DatabaseConfigError {
    #[error("unsupported database url: {0}")]
    UnsupportedUrl(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 执行 `detects supported database kinds` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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
