//! 健康检查路由模块，用于负载均衡、容器探针和本地 smoke test 判断 API 是否可用。

use axum::Json;
use serde::Serialize;

/// HealthResponse 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

/// 执行 `healthz` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
pub async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
