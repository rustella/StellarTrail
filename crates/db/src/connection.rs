//! 数据库连接模块，负责按配置创建 SeaORM DatabaseConnection。

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

use crate::DatabaseConfig;

/// 执行 `connect database` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
pub async fn connect_database(config: &DatabaseConfig) -> Result<DatabaseConnection, DbErr> {
    let mut url = config.url.clone();
    if url.starts_with("sqlite://") && !url.contains('?') && url != "sqlite::memory:" {
        url.push_str("?mode=rwc");
    }
    let mut options = ConnectOptions::new(url);
    options.sqlx_logging(false);
    Database::connect(options).await
}
