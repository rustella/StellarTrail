use std::sync::Arc;

use stellartrail_db::KnotRepository;

use crate::config::ApiConfig;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: ApiConfig,
    knot_repository: KnotRepository,
}

impl AppState {
    pub fn new(config: ApiConfig, knot_repository: KnotRepository) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config,
                knot_repository,
            }),
        }
    }

    pub fn config(&self) -> &ApiConfig {
        &self.inner.config
    }

    pub fn knot_repository(&self) -> &KnotRepository {
        &self.inner.knot_repository
    }
}
