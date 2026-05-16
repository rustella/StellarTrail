use std::{env, net::SocketAddr, path::PathBuf};

use stellartrail_db::{DatabaseConfig, DatabaseKind};

#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub app_env: String,
    pub host: String,
    pub port: u16,
    pub database: DatabaseConfig,
    pub content_assets_dir: PathBuf,
    pub media_base_url: String,
}

impl ApiConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "local".to_owned());
        let host = env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
        let port = env::var("APP_PORT")
            .unwrap_or_else(|_| "8080".to_owned())
            .parse::<u16>()?;
        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://stellartrail.db".to_owned());
        let content_assets_dir = env::var("CONTENT_ASSETS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("content/assets"));
        let media_base_url = env::var("MEDIA_BASE_URL").unwrap_or_else(|_| "/assets".to_owned());

        Ok(Self {
            app_env,
            host,
            port,
            database: DatabaseConfig::new(database_url)?,
            content_assets_dir,
            media_base_url,
        })
    }

    pub fn for_test(
        database_url: impl Into<String>,
        content_assets_dir: impl Into<PathBuf>,
        media_base_url: impl Into<String>,
    ) -> Self {
        Self {
            app_env: "test".to_owned(),
            host: "127.0.0.1".to_owned(),
            port: 0,
            database: DatabaseConfig::new(database_url.into()).expect("valid test database url"),
            content_assets_dir: content_assets_dir.into(),
            media_base_url: media_base_url.into(),
        }
    }

    pub fn bind_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("validated socket address")
    }

    pub fn database_kind(&self) -> DatabaseKind {
        self.database.kind
    }
}
