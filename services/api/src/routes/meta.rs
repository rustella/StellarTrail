//! 元信息路由模块，返回 API 版本和当前环境，便于前端与诊断工具确认后端实例。

use axum::{Json, extract::State};
use serde::Serialize;

use crate::state::AppState;

/// MetaResponse 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Serialize)]
pub struct MetaResponse {
    name: &'static str,
    env: String,
    database_kind: String,
}

/// 执行 `meta` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
pub async fn meta(State(state): State<AppState>) -> Json<MetaResponse> {
    Json(MetaResponse {
        name: "StellarTrail",
        env: state.config().app_env.clone(),
        database_kind: state.config().database_kind().as_str().to_owned(),
    })
}
