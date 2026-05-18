//! HTTP DTOs for administrator API usage statistics.
//!
//! Query parameters are intentionally limited to aggregate dimensions. The
//! response exposes counts by date, user id, route template, method, and status
//! only; it never includes raw paths, query strings, request bodies, headers, or
//! token material.

use serde::{Deserialize, Serialize};

/// Query parameters accepted by the administrator API usage list endpoint.
#[derive(Debug, Default, Deserialize)]
pub struct ApiUsageQueryParams {
    pub user_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub method: Option<String>,
    pub route: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

/// Paginated aggregate response for administrator API usage queries.
#[derive(Debug, Serialize)]
pub struct ApiUsageListResponse {
    pub items: Vec<ApiUsageSummaryResponse>,
    pub page: ApiUsagePageResponse,
}

/// One aggregate usage row.
#[derive(Debug, Serialize)]
pub struct ApiUsageSummaryResponse {
    pub bucket_date: String,
    pub user_id: Option<String>,
    pub method: String,
    pub route_pattern: String,
    pub status_code: i32,
    pub call_count: i64,
    pub first_called_at: String,
    pub last_called_at: String,
}

/// Pagination metadata for the aggregate response.
#[derive(Debug, Serialize)]
pub struct ApiUsagePageResponse {
    pub limit: u64,
    pub offset: u64,
    pub next_offset: Option<u64>,
}
