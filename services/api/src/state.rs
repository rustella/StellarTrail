//! 应用状态模块，集中保存配置、数据库连接、内容目录、微信客户端和缓存实例，供路由共享。

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use stellartrail_importer::ContentCatalog;

use crate::{
    cache::Cache,
    config::ApiConfig,
    services::wechat::{CurlWechatCodeSessionClient, WechatCodeSessionClient},
};

/// Axum 路由共享状态，内部使用 Arc 克隆以便在 handler 间低成本传递。
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

/// AppStateInner 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
struct AppStateInner {
    config: ApiConfig,
    db: DatabaseConnection,
    content: ContentCatalog,
    wechat_client: Arc<dyn WechatCodeSessionClient>,
    cache: Cache,
}

impl AppState {
    /// 执行 `new` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn new(config: ApiConfig, db: DatabaseConnection) -> Self {
        Self::new_with_content(config, db, ContentCatalog::default())
    }

    /// 执行 `new with cache` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn new_with_cache(config: ApiConfig, db: DatabaseConnection, cache: Cache) -> Self {
        Self::new_with_content_and_cache(config, db, ContentCatalog::default(), cache)
    }

    /// 执行 `new with content` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn new_with_content(
        config: ApiConfig,
        db: DatabaseConnection,
        content: ContentCatalog,
    ) -> Self {
        Self::new_with_content_and_cache(config, db, content, Cache::disabled())
    }

    /// 执行 `new with content and cache` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 执行 `new with wechat client` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 执行 `new with content and wechat client` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 执行 `new with content and wechat client and cache` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
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

    /// 执行 `config` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn config(&self) -> &ApiConfig {
        &self.inner.config
    }

    /// 执行 `db` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn db(&self) -> &DatabaseConnection {
        &self.inner.db
    }

    /// 执行 `content` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn content(&self) -> &ContentCatalog {
        &self.inner.content
    }

    /// 执行 `wechat client` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn wechat_client(&self) -> Arc<dyn WechatCodeSessionClient> {
        Arc::clone(&self.inner.wechat_client)
    }

    /// 执行 `cache` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    pub fn cache(&self) -> &Cache {
        &self.inner.cache
    }
}
