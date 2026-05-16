//! 路由聚合模块，组合所有 API 子路由并提供统一的 404 响应。

mod auth;
mod content;
mod gears;
mod health;
mod meta;

use axum::{Router, routing::get};

use crate::error::ApiError;
use crate::state::AppState;

/// 组合所有业务路由、健康检查和 404 fallback。
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/api/meta", get(meta::meta))
        .merge(auth::routes())
        .merge(content::routes())
        .merge(gears::routes())
        .fallback(not_found)
        .with_state(state)
}

/// 执行 `not found` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
async fn not_found() -> ApiError {
    ApiError::NotFound
}
