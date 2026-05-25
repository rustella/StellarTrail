//! API usage middleware, request-scoped user context, and asynchronous reporter.
//!
//! The middleware records only sanitized route templates and response metadata.
//! It does not read request bodies, query strings, authorization headers, token
//! values, cookies, IP addresses, or user agents. Events are sent with
//! `try_send` to a bounded background queue so statistics can be dropped without
//! changing the original API response.

use std::sync::{Arc, Mutex};

use axum::{
    extract::{MatchedPath, Request, State},
    http::Method,
    middleware::Next,
    response::Response,
};
use sea_orm::DatabaseConnection;
use stellartrail_db::repositories::{ApiUsageIncrement, ApiUsageRepository};
use time::{OffsetDateTime, format_description::well_known::Iso8601};
use tokio::sync::mpsc;

use crate::{routes::API_PREFIX_WITH_SLASH, state::AppState};

const API_USAGE_QUEUE_CAPACITY: usize = 1024;

/// Request-scoped authenticated user marker shared between extractors and middleware.
#[derive(Clone, Default)]
pub struct ApiUsageUserContext {
    user_id: Arc<Mutex<Option<String>>>,
}

impl ApiUsageUserContext {
    /// Stores the trusted user id after authentication has already succeeded.
    pub fn set_user_id(&self, user_id: impl Into<String>) {
        let mut guard = self
            .user_id
            .lock()
            .expect("api usage user context mutex should not be poisoned");
        *guard = Some(user_id.into());
    }

    /// Returns the trusted user id for this request, if authentication succeeded.
    pub fn user_id(&self) -> Option<String> {
        self.user_id
            .lock()
            .expect("api usage user context mutex should not be poisoned")
            .clone()
    }
}

/// Sanitized API usage event sent from middleware to the background worker.
#[derive(Clone, Debug)]
pub struct ApiUsageEvent {
    pub user_id: Option<String>,
    pub method: String,
    pub route_pattern: String,
    pub status_code: u16,
    pub occurred_at: OffsetDateTime,
}

impl ApiUsageEvent {
    fn into_increment(self) -> Result<ApiUsageIncrement, time::error::Format> {
        let occurred_at = self.occurred_at.format(&Iso8601::DEFAULT)?;
        Ok(ApiUsageIncrement {
            bucket_date: self.occurred_at.date().to_string(),
            user_id: self.user_id,
            method: self.method,
            route_pattern: self.route_pattern,
            status_code: i32::from(self.status_code),
            occurred_at,
            call_count: 1,
        })
    }
}

/// Best-effort asynchronous reporter backed by a bounded Tokio channel.
#[derive(Clone)]
pub struct ApiUsageReporter {
    sender: mpsc::Sender<ApiUsageEvent>,
}

impl ApiUsageReporter {
    /// Creates a reporter and starts the background aggregation worker.
    pub fn new(db: DatabaseConnection) -> Self {
        Self::with_capacity(db, API_USAGE_QUEUE_CAPACITY)
    }

    /// Creates a reporter with a caller-provided queue size for focused tests.
    pub fn with_capacity(db: DatabaseConnection, capacity: usize) -> Self {
        let (sender, mut receiver) = mpsc::channel::<ApiUsageEvent>(capacity.max(1));
        tokio::spawn(async move {
            let repo = ApiUsageRepository::new(db);
            while let Some(event) = receiver.recv().await {
                let increment = match event.into_increment() {
                    Ok(increment) => increment,
                    Err(error) => {
                        tracing::warn!(error = %error, "dropping api usage event with invalid timestamp");
                        continue;
                    }
                };
                if let Err(error) = repo.record_increment(&increment).await {
                    tracing::warn!(error = %error, "failed to persist api usage event");
                }
            }
        });
        Self { sender }
    }

    /// Attempts to enqueue an event without waiting for the database worker.
    pub fn try_report(&self, event: ApiUsageEvent) {
        if let Err(error) = self.sender.try_send(event) {
            tracing::warn!(error = %error, "dropping api usage event because reporter queue is unavailable");
        }
    }
}

/// Axum middleware that records one sanitized usage event after the response is built.
pub async fn track_api_usage(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let route_pattern = request
        .extensions()
        .get::<MatchedPath>()
        .map(|matched_path| matched_path.as_str().to_owned());
    let context = ApiUsageUserContext::default();
    request.extensions_mut().insert(context.clone());

    let response = next.run(request).await;
    if let Some(route_pattern) = route_pattern {
        if should_track(&method, &route_pattern) {
            state.api_usage_reporter().try_report(ApiUsageEvent {
                user_id: context.user_id(),
                method: method.as_str().to_owned(),
                route_pattern,
                status_code: response.status().as_u16(),
                occurred_at: OffsetDateTime::now_utc(),
            });
        }
    }
    response
}

fn should_track(method: &Method, route_pattern: &str) -> bool {
    method != Method::OPTIONS && route_pattern.starts_with(API_PREFIX_WITH_SLASH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_track_only_business_api_routes() {
        assert!(should_track(&Method::GET, "/api/v1/me/gears"));
        assert!(should_track(&Method::POST, "/api/v1/auth/login"));
        assert!(!should_track(&Method::GET, "/healthz"));
        assert!(!should_track(&Method::GET, "/api/me/gears"));
        assert!(!should_track(&Method::OPTIONS, "/api/v1/me/gears"));
    }

    #[test]
    fn event_increment_keeps_user_id_and_route_pattern_only() {
        let event = ApiUsageEvent {
            user_id: Some("user-1".to_owned()),
            method: "GET".to_owned(),
            route_pattern: "/api/v1/me/gears/:id".to_owned(),
            status_code: 200,
            occurred_at: OffsetDateTime::from_unix_timestamp(1_779_120_000).unwrap(),
        };

        let increment = event.into_increment().unwrap();
        assert_eq!(increment.user_id.as_deref(), Some("user-1"));
        assert_eq!(increment.route_pattern, "/api/v1/me/gears/:id");
        assert_eq!(increment.status_code, 200);
        assert_eq!(increment.call_count, 1);
    }
}
