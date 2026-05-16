use std::sync::Arc;

use sea_orm::DatabaseConnection;
use stellartrail_importer::ContentCatalog;

use crate::{
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
        Self::new_with_content_and_wechat_client(
            config,
            db,
            content,
            Arc::new(CurlWechatCodeSessionClient),
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
        Self {
            inner: Arc::new(AppStateInner {
                config,
                db,
                content,
                wechat_client,
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
}
