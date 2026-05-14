use std::{env, net::SocketAddr};

use stellartrail_db::{DatabaseConfig, DatabaseKind};

#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub app_env: String,
    pub host: String,
    pub port: u16,
    pub database: DatabaseConfig,
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

        Ok(Self {
            app_env,
            host,
            port,
            database: DatabaseConfig::new(database_url)?,
        })
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
