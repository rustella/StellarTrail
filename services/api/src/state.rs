//! Application state module that stores configuration, database connection, WeChat client, cache, object storage, email sender, and DB-backed public API helpers for routes.

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use stellartrail_db::repositories::{
    DisclaimerAcceptanceRepository, GearTemplateRepository, KnotRepository, SkillFavoriteRepository,
};

use crate::{
    api_usage::ApiUsageReporter,
    cache::Cache,
    config::ApiConfig,
    email::{EmailSender, NoopEmailSender},
    object_store::{InMemoryObjectStore, ObjectStore},
    services::{
        public_response_cache::InMemoryPublicResponseCache,
        rate_limit_service::InMemoryRateLimiter,
        wechat::{HttpWechatCodeSessionClient, WechatCodeSessionClient},
    },
};

/// Shared Axum route state that uses Arc internally for low-cost cloning across handlers.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: ApiConfig,
    db: DatabaseConnection,
    wechat_client: Arc<dyn WechatCodeSessionClient>,
    cache: Cache,
    object_store: Arc<dyn ObjectStore>,
    email_sender: Arc<dyn EmailSender>,
    disclaimer_acceptance_repository: DisclaimerAcceptanceRepository,
    knot_repository: KnotRepository,
    skill_favorite_repository: SkillFavoriteRepository,
    gear_template_repository: GearTemplateRepository,
    rate_limiter: InMemoryRateLimiter,
    public_response_cache: InMemoryPublicResponseCache,
    api_usage_reporter: ApiUsageReporter,
}

impl AppState {
    /// Creates app state with disabled cache and test in-memory object store.
    pub fn new(config: ApiConfig, db: DatabaseConnection) -> Self {
        Self::new_with_cache(config, db, Cache::disabled())
    }

    /// Creates app state with a custom cache and test in-memory object store.
    pub fn new_with_cache(config: ApiConfig, db: DatabaseConnection, cache: Cache) -> Self {
        Self::new_with_wechat_client_cache_object_store_and_email_sender(
            config,
            db,
            Arc::new(HttpWechatCodeSessionClient),
            cache,
            Arc::new(InMemoryObjectStore::default()),
            Arc::new(NoopEmailSender),
        )
    }

    /// Creates app state with custom cache and custom object store.
    pub fn new_with_cache_and_object_store(
        config: ApiConfig,
        db: DatabaseConnection,
        cache: Cache,
        object_store: Arc<dyn ObjectStore>,
    ) -> Self {
        Self::new_with_wechat_client_cache_object_store_and_email_sender(
            config,
            db,
            Arc::new(HttpWechatCodeSessionClient),
            cache,
            object_store,
            Arc::new(NoopEmailSender),
        )
    }

    /// Creates app state with a custom WeChat client for auth tests.
    pub fn new_with_wechat_client(
        config: ApiConfig,
        db: DatabaseConnection,
        wechat_client: Arc<dyn WechatCodeSessionClient>,
    ) -> Self {
        Self::new_with_wechat_client_cache_object_store_and_email_sender(
            config,
            db,
            wechat_client,
            Cache::disabled(),
            Arc::new(InMemoryObjectStore::default()),
            Arc::new(NoopEmailSender),
        )
    }

    /// Creates app state with default dependencies and custom transactional email sender.
    pub fn new_with_email_sender(
        config: ApiConfig,
        db: DatabaseConnection,
        email_sender: Arc<dyn EmailSender>,
    ) -> Self {
        Self::new_with_wechat_client_cache_object_store_and_email_sender(
            config,
            db,
            Arc::new(HttpWechatCodeSessionClient),
            Cache::disabled(),
            Arc::new(InMemoryObjectStore::default()),
            email_sender,
        )
    }

    /// Creates app state with every dependency injected explicitly except email sender.
    pub fn new_with_wechat_client_cache_and_object_store(
        config: ApiConfig,
        db: DatabaseConnection,
        wechat_client: Arc<dyn WechatCodeSessionClient>,
        cache: Cache,
        object_store: Arc<dyn ObjectStore>,
    ) -> Self {
        Self::new_with_wechat_client_cache_object_store_and_email_sender(
            config,
            db,
            wechat_client,
            cache,
            object_store,
            Arc::new(NoopEmailSender),
        )
    }

    /// Creates app state with every dependency injected explicitly, including the transactional email sender.
    pub fn new_with_wechat_client_cache_object_store_and_email_sender(
        config: ApiConfig,
        db: DatabaseConnection,
        wechat_client: Arc<dyn WechatCodeSessionClient>,
        cache: Cache,
        object_store: Arc<dyn ObjectStore>,
        email_sender: Arc<dyn EmailSender>,
    ) -> Self {
        let disclaimer_acceptance_repository = DisclaimerAcceptanceRepository::new(db.clone());
        let knot_repository = KnotRepository::new(db.clone());
        let skill_favorite_repository = SkillFavoriteRepository::new(db.clone());
        let gear_template_repository = GearTemplateRepository::new(db.clone());
        let api_usage_reporter = ApiUsageReporter::new(db.clone());
        Self {
            inner: Arc::new(AppStateInner {
                config,
                db,
                wechat_client,
                cache,
                object_store,
                email_sender,
                disclaimer_acceptance_repository,
                knot_repository,
                skill_favorite_repository,
                gear_template_repository,
                rate_limiter: InMemoryRateLimiter::default(),
                public_response_cache: InMemoryPublicResponseCache::default(),
                api_usage_reporter,
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

    /// Returns the transactional email sender.
    pub fn email_sender(&self) -> Arc<dyn EmailSender> {
        Arc::clone(&self.inner.email_sender)
    }

    /// Returns the DB-backed current-user disclaimer acceptance repository.
    pub fn disclaimer_acceptance_repository(&self) -> &DisclaimerAcceptanceRepository {
        &self.inner.disclaimer_acceptance_repository
    }

    /// Returns the DB-backed public knot repository.
    pub fn knot_repository(&self) -> &KnotRepository {
        &self.inner.knot_repository
    }

    /// Returns the DB-backed current-user skill favorite repository.
    pub fn skill_favorite_repository(&self) -> &SkillFavoriteRepository {
        &self.inner.skill_favorite_repository
    }

    /// Returns the DB-backed public gear template repository.
    pub fn gear_template_repository(&self) -> &GearTemplateRepository {
        &self.inner.gear_template_repository
    }

    /// Returns the in-memory fallback global API limiter.
    pub fn rate_limiter(&self) -> &InMemoryRateLimiter {
        &self.inner.rate_limiter
    }

    /// Returns the in-memory fallback public response cache.
    pub fn public_response_cache(&self) -> &InMemoryPublicResponseCache {
        &self.inner.public_response_cache
    }

    /// Returns the best-effort API usage reporter.
    pub fn api_usage_reporter(&self) -> &ApiUsageReporter {
        &self.inner.api_usage_reporter
    }
}
