//! Application state module that stores configuration, database connection, content catalog, WeChat client, cache, object storage, and knot repository for routes.

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use stellartrail_db::repositories::KnotRepository;
use stellartrail_importer::ContentCatalog;

use crate::{
    cache::Cache,
    config::ApiConfig,
    object_store::{InMemoryObjectStore, ObjectStore},
    services::wechat::{CurlWechatCodeSessionClient, WechatCodeSessionClient},
};

/// Shared Axum route state that uses Arc internally for low-cost cloning across handlers.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: ApiConfig,
    db: DatabaseConnection,
    content: ContentCatalog,
    wechat_client: Arc<dyn WechatCodeSessionClient>,
    cache: Cache,
    object_store: Arc<dyn ObjectStore>,
    knot_repository: KnotRepository,
}

impl AppState {
    /// Creates app state with default content, disabled cache, and test in-memory object store.
    pub fn new(config: ApiConfig, db: DatabaseConnection) -> Self {
        Self::new_with_content(config, db, ContentCatalog::default())
    }

    /// Creates app state with default content, a custom cache, and test in-memory object store.
    pub fn new_with_cache(config: ApiConfig, db: DatabaseConnection, cache: Cache) -> Self {
        Self::new_with_content_and_cache(config, db, ContentCatalog::default(), cache)
    }

    /// Creates app state with default content, custom cache, and custom object store.
    pub fn new_with_cache_and_object_store(
        config: ApiConfig,
        db: DatabaseConnection,
        cache: Cache,
        object_store: Arc<dyn ObjectStore>,
    ) -> Self {
        Self::new_with_content_and_wechat_client_cache_and_object_store(
            config,
            db,
            ContentCatalog::default(),
            Arc::new(CurlWechatCodeSessionClient),
            cache,
            object_store,
        )
    }

    /// Creates app state with provided content, disabled cache, and test in-memory object store.
    pub fn new_with_content(
        config: ApiConfig,
        db: DatabaseConnection,
        content: ContentCatalog,
    ) -> Self {
        Self::new_with_content_and_cache(config, db, content, Cache::disabled())
    }

    /// Creates app state with provided content/cache and test in-memory object store.
    pub fn new_with_content_and_cache(
        config: ApiConfig,
        db: DatabaseConnection,
        content: ContentCatalog,
        cache: Cache,
    ) -> Self {
        Self::new_with_content_and_wechat_client_cache_and_object_store(
            config,
            db,
            content,
            Arc::new(CurlWechatCodeSessionClient),
            cache,
            Arc::new(InMemoryObjectStore::default()),
        )
    }

    /// Creates app state with a custom WeChat client for auth tests.
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

    /// Creates app state with custom content and WeChat client.
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

    /// Creates app state with custom content, WeChat client, cache, and test in-memory object store.
    pub fn new_with_content_and_wechat_client_and_cache(
        config: ApiConfig,
        db: DatabaseConnection,
        content: ContentCatalog,
        wechat_client: Arc<dyn WechatCodeSessionClient>,
        cache: Cache,
    ) -> Self {
        Self::new_with_content_and_wechat_client_cache_and_object_store(
            config,
            db,
            content,
            wechat_client,
            cache,
            Arc::new(InMemoryObjectStore::default()),
        )
    }

    /// Creates app state with every dependency injected explicitly.
    pub fn new_with_content_and_wechat_client_cache_and_object_store(
        config: ApiConfig,
        db: DatabaseConnection,
        content: ContentCatalog,
        wechat_client: Arc<dyn WechatCodeSessionClient>,
        cache: Cache,
        object_store: Arc<dyn ObjectStore>,
    ) -> Self {
        let knot_repository = KnotRepository::new(db.clone(), config.media_base_url.clone());
        Self {
            inner: Arc::new(AppStateInner {
                config,
                db,
                content,
                wechat_client,
                cache,
                object_store,
                knot_repository,
            }),
        }
    }

    /// Returns runtime configuration.
    pub fn config(&self) -> &ApiConfig {
        &self.inner.config
    }

    /// Returns the database connection.
    pub fn db(&self) -> &DatabaseConnection {
        &self.inner.db
    }

    /// Returns the content catalog.
    pub fn content(&self) -> &ContentCatalog {
        &self.inner.content
    }

    /// Returns the WeChat code2session client.
    pub fn wechat_client(&self) -> Arc<dyn WechatCodeSessionClient> {
        Arc::clone(&self.inner.wechat_client)
    }

    /// Returns the optional cache facade.
    pub fn cache(&self) -> &Cache {
        &self.inner.cache
    }

    /// Returns the object storage backend.
    pub fn object_store(&self) -> Arc<dyn ObjectStore> {
        Arc::clone(&self.inner.object_store)
    }

    /// Returns the knot repository.
    pub fn knot_repository(&self) -> &KnotRepository {
        &self.inner.knot_repository
    }
}
