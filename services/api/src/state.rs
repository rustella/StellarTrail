use std::sync::Arc;

use crate::config::ApiConfig;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: ApiConfig,
}

impl AppState {
    pub fn new(config: ApiConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config }),
        }
    }

    pub fn config(&self) -> &ApiConfig {
        &self.inner.config
    }
}
