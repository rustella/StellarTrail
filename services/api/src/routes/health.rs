//! Health-check route module used by load balancers, container probes, and local smoke tests to determine API availability.

use axum::Json;
use serde::Serialize;

/// Stable data boundary for `HealthResponse`, exposed by or reused within this module.
#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

/// Runs the `healthz` server-side flow while preserving input validation, error propagation, and state invariants.
pub async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
