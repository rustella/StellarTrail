use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::config::ApiConfig;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: ApiConfig,
    db: DatabaseConnection,
}

impl AppState {
    pub fn new(config: ApiConfig, db: DatabaseConnection) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config, db }),
        }
    }

    pub fn config(&self) -> &ApiConfig {
        &self.inner.config
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.inner.db
    }
}
