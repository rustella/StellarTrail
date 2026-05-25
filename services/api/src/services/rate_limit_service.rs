//! Global application route fixed-window rate limiting middleware and helpers.

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, HeaderValue, Method, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use stellartrail_db::repositories::{AuthRepository, hash_token};

use crate::{config::RateLimitConfig, error::ApiError, state::AppState};

/// Rate-limit dimension used for stable Redis/in-memory keys and response headers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RateLimitDimension {
    Ip,
    User,
}

impl RateLimitDimension {
    fn as_key(self) -> &'static str {
        match self {
            Self::Ip => "ip",
            Self::User => "user",
        }
    }
}

/// Decision returned after checking one global fixed-window rate-limit bucket.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RateLimitDecision {
    pub allowed: bool,
    pub dimension: RateLimitDimension,
    pub limit: u64,
    pub remaining: u64,
    pub retry_after_seconds: u64,
    pub reset_unix_seconds: u64,
}

/// In-memory fallback limiter used when Redis is disabled or unavailable.
#[derive(Clone, Default)]
pub struct InMemoryRateLimiter {
    inner: Arc<Mutex<HashMap<String, Counter>>>,
}

#[derive(Clone, Copy)]
struct Counter {
    count: u64,
    expires_at: Instant,
    reset_unix_seconds: u64,
}

impl InMemoryRateLimiter {
    /// Increments one fixed-window key and returns the new count plus reset epoch.
    pub fn increment(&self, key: &str, window: Duration) -> (u64, u64) {
        let now = Instant::now();
        let reset_unix_seconds = unix_now().saturating_add(window.as_secs());
        let mut inner = self.inner.lock().expect("global limiter mutex poisoned");
        inner.retain(|_, counter| counter.expires_at > now);
        let counter = inner.entry(key.to_owned()).or_insert(Counter {
            count: 0,
            expires_at: now + window,
            reset_unix_seconds,
        });
        if counter.expires_at <= now {
            *counter = Counter {
                count: 0,
                expires_at: now + window,
                reset_unix_seconds,
            };
        }
        counter.count = counter.count.saturating_add(1);
        (counter.count, counter.reset_unix_seconds)
    }
}

/// Axum middleware that enforces global IP and authenticated-user rate limits before handlers run.
pub async fn enforce_global_rate_limit(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if !state.config().rate_limit.enabled || request.method() == Method::OPTIONS {
        return next.run(request).await;
    }

    let headers = request.headers().clone();
    let client_ip = client_ip_from_request(
        &headers,
        request.extensions().get::<ConnectInfo<SocketAddr>>(),
        &state.config().rate_limit,
    )
    .to_string();
    let bearer = bearer_token(&headers).map(ToOwned::to_owned);

    let ip_decision = check_global_rate_limit(
        &state,
        RateLimitDimension::Ip,
        &client_ip,
        state.config().rate_limit.max_requests_per_ip,
    )
    .await;
    if !ip_decision.allowed {
        return rate_limited_response(&ip_decision);
    }

    if let Some(token) = bearer {
        let token_hash = hash_token(&token);
        match AuthRepository::new(state.db().clone())
            .find_user_by_token_hash(&token_hash)
            .await
        {
            Ok(Some(user)) => {
                let user_decision = check_global_rate_limit(
                    &state,
                    RateLimitDimension::User,
                    &user.id,
                    state.config().rate_limit.max_requests_per_user,
                )
                .await;
                if !user_decision.allowed {
                    return rate_limited_response(&user_decision);
                }
            }
            Ok(None) => {}
            Err(error) => return ApiError::from(error).into_response(),
        }
    }

    next.run(request).await
}

/// Checks one global fixed-window rate-limit bucket using Redis first and an in-memory fallback.
pub async fn check_global_rate_limit(
    state: &AppState,
    dimension: RateLimitDimension,
    subject: &str,
    limit: u64,
) -> RateLimitDecision {
    let config = &state.config().rate_limit;
    let window = Duration::from_secs(config.window_seconds);
    let reset_unix_seconds = current_window_reset(config.window_seconds);

    if !config.enabled {
        return RateLimitDecision {
            allowed: true,
            dimension,
            limit,
            remaining: limit,
            retry_after_seconds: 0,
            reset_unix_seconds,
        };
    }

    let window_start = unix_now() / config.window_seconds;
    let key = format!(
        "{}:rate-limit:global:{}:{}:{window_start}",
        state.config().redis_cache.key_prefix,
        dimension.as_key(),
        subject,
    );
    let count = match state.cache().increment_with_ttl(&key, window).await {
        Some(count) => count,
        None => {
            let (count, _) = state.rate_limiter().increment(&key, window);
            count
        }
    };
    let allowed = count <= limit;
    let remaining = limit.saturating_sub(count);
    RateLimitDecision {
        allowed,
        dimension,
        limit,
        remaining,
        retry_after_seconds: retry_after(reset_unix_seconds),
        reset_unix_seconds,
    }
}

/// Resolves the rate-limit client IP, trusting forwarded headers only for configured proxy CIDRs.
pub fn client_ip_from_request(
    headers: &HeaderMap,
    connect_info: Option<&ConnectInfo<SocketAddr>>,
    config: &RateLimitConfig,
) -> IpAddr {
    let direct_ip = connect_info
        .map(|ConnectInfo(addr)| addr.ip())
        .unwrap_or(IpAddr::from([0, 0, 0, 0]));

    if config.trust_proxy_headers && is_trusted_proxy(direct_ip, &config.trusted_proxy_cidrs) {
        if let Some(forwarded) = first_forwarded_for(headers) {
            return forwarded;
        }
    }

    direct_ip
}

fn rate_limited_response(decision: &RateLimitDecision) -> Response {
    let mut response = ApiError::RateLimited {
        retry_after_seconds: decision.retry_after_seconds,
    }
    .into_response();
    insert_u64_header(response.headers_mut(), "x-ratelimit-limit", decision.limit);
    insert_u64_header(
        response.headers_mut(),
        "x-ratelimit-remaining",
        decision.remaining,
    );
    insert_u64_header(
        response.headers_mut(),
        "x-ratelimit-reset",
        decision.reset_unix_seconds,
    );
    response
}

fn insert_u64_header(headers: &mut HeaderMap, name: &'static str, value: u64) {
    if let Ok(value) = HeaderValue::from_str(&value.to_string()) {
        headers.insert(name, value);
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
}

fn first_forwarded_for(headers: &HeaderMap) -> Option<IpAddr> {
    let raw = headers.get("x-forwarded-for")?.to_str().ok()?;
    if raw.len() > 512 {
        return None;
    }
    raw.split(',').next()?.trim().parse::<IpAddr>().ok()
}

fn is_trusted_proxy(ip: IpAddr, cidrs: &[String]) -> bool {
    cidrs.iter().any(|cidr| cidr_matches(ip, cidr))
}

fn cidr_matches(ip: IpAddr, cidr: &str) -> bool {
    let Some((base, prefix)) = cidr.split_once('/') else {
        return cidr.parse::<IpAddr>().is_ok_and(|base| base == ip);
    };
    let Ok(prefix) = prefix.parse::<u8>() else {
        return false;
    };
    match (ip, base.parse::<IpAddr>()) {
        (IpAddr::V4(ip), Ok(IpAddr::V4(base))) if prefix <= 32 => {
            let mask = if prefix == 0 {
                0
            } else {
                u32::MAX << (32 - prefix)
            };
            (u32::from(ip) & mask) == (u32::from(base) & mask)
        }
        (IpAddr::V6(ip), Ok(IpAddr::V6(base))) if prefix <= 128 => {
            let mask = if prefix == 0 {
                0
            } else {
                u128::MAX << (128 - prefix)
            };
            (u128::from(ip) & mask) == (u128::from(base) & mask)
        }
        _ => false,
    }
}

fn current_window_reset(window_seconds: u64) -> u64 {
    let now = unix_now();
    ((now / window_seconds) + 1).saturating_mul(window_seconds)
}

fn retry_after(reset_unix_seconds: u64) -> u64 {
    reset_unix_seconds.saturating_sub(unix_now()).max(1)
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_ip_ignores_forwarded_for_without_trusted_proxy() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.1"));
        let config = RateLimitConfig::default();
        let connect_info = ConnectInfo(SocketAddr::from(([198, 51, 100, 1], 12345)));

        assert_eq!(
            client_ip_from_request(&headers, Some(&connect_info), &config),
            IpAddr::from([198, 51, 100, 1])
        );
    }

    #[test]
    fn client_ip_uses_forwarded_for_from_trusted_proxy() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.1, 198.51.100.2"),
        );
        let config = RateLimitConfig {
            trust_proxy_headers: true,
            trusted_proxy_cidrs: vec!["172.16.0.0/12".to_owned()],
            ..RateLimitConfig::default()
        };
        let connect_info = ConnectInfo(SocketAddr::from(([172, 16, 1, 1], 12345)));

        assert_eq!(
            client_ip_from_request(&headers, Some(&connect_info), &config),
            IpAddr::from([203, 0, 113, 1])
        );
    }

    #[test]
    fn cidr_matches_ipv4_and_ipv6_ranges() {
        assert!(cidr_matches("172.16.1.1".parse().unwrap(), "172.16.0.0/12"));
        assert!(!cidr_matches(
            "198.51.100.1".parse().unwrap(),
            "172.16.0.0/12"
        ));
        assert!(cidr_matches(
            "2001:db8::1".parse().unwrap(),
            "2001:db8::/32"
        ));
    }
}
