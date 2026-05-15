use std::sync::Arc;

use sea_orm::DatabaseConnection;
use stellartrail_importer::ContentCatalog;

use crate::config::ApiConfig;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: ApiConfig,
    db: DatabaseConnection,
    content: ContentCatalog,
}

impl AppState {
    pub fn new(config: ApiConfig, db: DatabaseConnection) -> Self {
        Self::new_with_content(config, db, ContentCatalog::default())
    }

    pub fn new_with_content(
        config: ApiConfig,
        db: DatabaseConnection,
        content: ContentCatalog,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config,
                db,
                content,
            }),
        }
    }

    pub fn config(&self) -> &ApiConfig {
        &self.inner.config
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.inner.db
    }

    pub fn content(&self) -> &ContentCatalog {
        &self.inner.content
    }
}
