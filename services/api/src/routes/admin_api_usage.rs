//! Administrator API usage statistics routes.
//!
//! The endpoint returns aggregate counters only. It is protected by
//! database-backed administrator roles and accepts a small set of bounded filters
//! so callers cannot turn statistics into a raw request-log search tool.

use axum::{Json, Router, extract::Query, extract::State, routing::get};
use stellartrail_db::repositories::{ApiUsageQuery, ApiUsageRecord, ApiUsageRepository};
use time::{Duration, Month, OffsetDateTime};

use crate::{
    dto::api_usage::{
        ApiUsageListResponse, ApiUsagePageResponse, ApiUsageQueryParams, ApiUsageSummaryResponse,
    },
    error::ApiError,
    extractors::AuthenticatedUser,
    services::admin_service,
    state::AppState,
};

use super::{API_PREFIX, API_PREFIX_WITH_SLASH};

const DEFAULT_LOOKBACK_DAYS: i64 = 30;
const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 100;

/// Builds administrator-only API usage routes.
pub fn routes() -> Router<AppState> {
    Router::new().route("/admin/api-usage", get(list_api_usage))
}

async fn list_api_usage(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(params): Query<ApiUsageQueryParams>,
) -> Result<Json<ApiUsageListResponse>, ApiError> {
    admin_service::ensure_admin(&state, &user).await?;
    let query = normalize_query(params)?;
    let limit = u64::try_from(query.limit).map_err(ApiError::internal)?;
    let offset = u64::try_from(query.offset).map_err(ApiError::internal)?;
    let rows = ApiUsageRepository::new(state.db().clone())
        .list(&query)
        .await?;
    let returned = rows.len() as u64;
    Ok(Json(ApiUsageListResponse {
        items: rows
            .into_iter()
            .map(ApiUsageSummaryResponse::from)
            .collect(),
        page: ApiUsagePageResponse {
            limit,
            offset,
            next_offset: (returned == limit).then_some(offset + limit),
        },
    }))
}

fn normalize_query(params: ApiUsageQueryParams) -> Result<ApiUsageQuery, ApiError> {
    let now = OffsetDateTime::now_utc();
    let default_from = (now - Duration::days(DEFAULT_LOOKBACK_DAYS - 1))
        .date()
        .to_string();
    let default_to = now.date().to_string();
    let from_date = match params.from {
        Some(value) => validate_date("from", value)?,
        None => default_from,
    };
    let to_date = match params.to {
        Some(value) => validate_date("to", value)?,
        None => default_to,
    };
    if from_date > to_date {
        return Err(ApiError::invalid_query_parameter(
            "from",
            "from must be before or equal to to".to_owned(),
        ));
    }

    let method = params.method.map(validate_method).transpose()?;
    let route_pattern = params.route.map(validate_route_pattern).transpose()?;
    let user_id = params
        .user_id
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = params.offset.unwrap_or(0);

    Ok(ApiUsageQuery {
        from_date,
        to_date,
        user_id,
        method,
        route_pattern,
        limit: i64::try_from(limit).map_err(ApiError::internal)?,
        offset: i64::try_from(offset).map_err(ApiError::internal)?,
    })
}

fn validate_date(field: &'static str, value: String) -> Result<String, ApiError> {
    let value = value.trim();
    let valid_shape = value.len() == 10
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value
            .chars()
            .enumerate()
            .all(|(index, ch)| index == 4 || index == 7 || ch.is_ascii_digit());
    if !valid_shape {
        return Err(ApiError::invalid_query_parameter(
            field,
            "date must use YYYY-MM-DD".to_owned(),
        ));
    }
    let year = value[0..4]
        .parse::<i32>()
        .map_err(|_| ApiError::invalid_query_parameter(field, "year is invalid".to_owned()))?;
    let month_number = value[5..7]
        .parse::<u8>()
        .map_err(|_| ApiError::invalid_query_parameter(field, "month is invalid".to_owned()))?;
    let day = value[8..10]
        .parse::<u8>()
        .map_err(|_| ApiError::invalid_query_parameter(field, "day is invalid".to_owned()))?;
    let month = Month::try_from(month_number)
        .map_err(|_| ApiError::invalid_query_parameter(field, "month is invalid".to_owned()))?;
    time::Date::from_calendar_date(year, month, day)
        .map_err(|_| ApiError::invalid_query_parameter(field, "date is invalid".to_owned()))?;
    Ok(value.to_owned())
}

fn validate_method(value: String) -> Result<String, ApiError> {
    let method = value.trim().to_ascii_uppercase();
    const ALLOWED: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE"];
    if ALLOWED.contains(&method.as_str()) {
        Ok(method)
    } else {
        Err(ApiError::invalid_query_parameter(
            "method",
            "method is not supported".to_owned(),
        ))
    }
}

fn validate_route_pattern(value: String) -> Result<String, ApiError> {
    let route = value.trim();
    if route.starts_with(API_PREFIX_WITH_SLASH) && !route.contains('?') && route.len() <= 200 {
        Ok(route.to_owned())
    } else {
        Err(ApiError::invalid_query_parameter(
            "route",
            format!("route must be an {API_PREFIX} route pattern without query string"),
        ))
    }
}

impl From<ApiUsageRecord> for ApiUsageSummaryResponse {
    fn from(record: ApiUsageRecord) -> Self {
        Self {
            bucket_date: record.bucket_date,
            user_id: record.user_id,
            method: record.method,
            route_pattern: record.route_pattern,
            status_code: record.status_code,
            call_count: record.call_count,
            first_called_at: record.first_called_at,
            last_called_at: record.last_called_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_query_defaults_and_bounds_limit() {
        let query = normalize_query(ApiUsageQueryParams {
            limit: Some(1_000),
            offset: Some(5),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(query.limit, 100);
        assert_eq!(query.offset, 5);
    }

    #[test]
    fn normalize_query_rejects_raw_query_in_route_filter() {
        let error = normalize_query(ApiUsageQueryParams {
            route: Some("/api/v1/me/gears?token=x".to_owned()),
            ..Default::default()
        })
        .unwrap_err();
        match error {
            ApiError::BadRequestWithCode { parameter, .. } => {
                assert_eq!(parameter.as_deref(), Some("route"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
