use std::sync::Arc;

use sea_orm::DatabaseConnection;
use stellartrail_importer::ContentCatalog;

use crate::{
    cache::Cache,
    config::ApiConfig,
    services::wechat::{CurlWechatCodeSessionClient, WechatCodeSessionClient},
};

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
}

impl AppState {
    pub fn new(config: ApiConfig, db: DatabaseConnection) -> Self {
        Self::new_with_content(config, db, ContentCatalog::default())
    }

    pub fn new_with_cache(config: ApiConfig, db: DatabaseConnection, cache: Cache) -> Self {
        Self::new_with_content_and_cache(config, db, ContentCatalog::default(), cache)
    }

    pub fn new_with_content(
        config: ApiConfig,
        db: DatabaseConnection,
        content: ContentCatalog,
    ) -> Self {
        Self::new_with_content_and_cache(config, db, content, Cache::disabled())
    }

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

    pub fn config(&self) -> &ApiConfig {
        &self.inner.config
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.inner.db
    }

    pub fn content(&self) -> &ContentCatalog {
        &self.inner.content
    }

    pub fn wechat_client(&self) -> Arc<dyn WechatCodeSessionClient> {
        Arc::clone(&self.inner.wechat_client)
    }

    pub fn cache(&self) -> &Cache {
        &self.inner.cache
    }
}
