//! 数据库迁移命令入口，供本地开发、部署流水线或一次性维护任务显式执行 schema 升降级。

use sea_orm_migration::prelude::MigratorTrait;
use stellartrail_api::config::ApiConfig;
use stellartrail_db::connect_database;
use stellartrail_migration::Migrator;

/// 执行 `main` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let command = std::env::args().nth(1).unwrap_or_else(|| "up".to_owned());
    let config = ApiConfig::from_env()?;
    let db = connect_database(&config.database).await?;
    match command.as_str() {
        "up" => Migrator::up(&db, None).await?,
        "down" => Migrator::down(&db, None).await?,
        "fresh" => Migrator::fresh(&db).await?,
        other => anyhow::bail!("unsupported migrate command: {other}"),
    }
    Ok(())
}
