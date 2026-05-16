//! Application state module that stores configuration, database connection, content catalog, WeChat client, and cache instance for routes.

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use stellartrail_importer::ContentCatalog;

use crate::{
    cache::Cache,
    config::ApiConfig,
    services::wechat::{CurlWechatCodeSessionClient, WechatCodeSessionClient},
};

/// Shared Axum route state that uses Arc internally for low-cost cloning across handlers.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

/// Stable data boundary for `AppStateInner`, exposed by or reused within this module.
struct AppStateInner {
    config: ApiConfig,
    db: DatabaseConnection,
    content: ContentCatalog,
    wechat_client: Arc<dyn WechatCodeSessionClient>,
    cache: Cache,
}

impl AppState {
    /// Runs the `new` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn new(config: ApiConfig, db: DatabaseConnection) -> Self {
        Self::new_with_content(config, db, ContentCatalog::default())
    }

    /// Runs the `new with cache` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn new_with_cache(config: ApiConfig, db: DatabaseConnection, cache: Cache) -> Self {
        Self::new_with_content_and_cache(config, db, ContentCatalog::default(), cache)
    }

    /// Runs the `new with content` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn new_with_content(
        config: ApiConfig,
        db: DatabaseConnection,
        content: ContentCatalog,
    ) -> Self {
        Self::new_with_content_and_cache(config, db, content, Cache::disabled())
    }

    /// Runs the `new with content and cache` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn new_with_content_and_cache(
        config: ApiConfig,
        db: DatabaseConnection,
        content: ContentCatalog,
        cache: Cache,
    ) -> Self {
        Self::new_with_content_and_wechat_client_and_cache(
            config,
            db,
            content,
            Arc::new(CurlWechatCodeSessionClient),
            cache,
        )
    }

    /// Runs the `new with wechat client` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn new_with_wechat_client(
        config: ApiConfig,
        db: DatabaseConnection,
        wechat_client: Arc<dyn WechatCodeSessionClient>,
    ) -> Self {
        Self::new_with_content_and_wechat_client(
            config,
            db,
            ContentCatalog::default(),
            wechat_client,
        )
    }

    /// Runs the `new with content and wechat client` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn new_with_content_and_wechat_client(
        config: ApiConfig,
        db: DatabaseConnection,
        content: ContentCatalog,
        wechat_client: Arc<dyn WechatCodeSessionClient>,
    ) -> Self {
        Self::new_with_content_and_wechat_client_and_cache(
            config,
            db,
            content,
            wechat_client,
            Cache::disabled(),
        )
    }

    /// Runs the `new with content and wechat client and cache` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn new_with_content_and_wechat_client_and_cache(
        config: ApiConfig,
        db: DatabaseConnection,
        content: ContentCatalog,
        wechat_client: Arc<dyn WechatCodeSessionClient>,
        cache: Cache,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config,
                db,
                content,
                wechat_client,
                cache,
            }),
        }
    }

    /// Runs the `config` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn config(&self) -> &ApiConfig {
        &self.inner.config
    }

    /// Runs the `db` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn db(&self) -> &DatabaseConnection {
        &self.inner.db
    }

    /// Runs the `content` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn content(&self) -> &ContentCatalog {
        &self.inner.content
    }

    /// Runs the `wechat client` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn wechat_client(&self) -> Arc<dyn WechatCodeSessionClient> {
        Arc::clone(&self.inner.wechat_client)
    }

    /// Runs the `cache` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn cache(&self) -> &Cache {
        &self.inner.cache
    }
}
