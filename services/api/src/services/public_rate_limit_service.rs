//! Public unauthenticated API fixed-window rate limiting helpers.

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use axum::extract::ConnectInfo;
use axum::http::HeaderMap;

use crate::{config::PublicApiConfig, state::AppState};

/// Decision returned after checking a public API fixed-window rate limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RateLimitDecision {
    pub allowed: bool,
    pub limit: u64,
    pub remaining: u64,
    pub retry_after_seconds: u64,
    pub reset_unix_seconds: u64,
}

/// In-memory fallback limiter used when Redis is disabled or unavailable.
#[derive(Clone, Default)]
pub struct InMemoryPublicRateLimiter {
    inner: Arc<Mutex<HashMap<String, Counter>>>,
}

#[derive(Clone, Copy)]
struct Counter {
    count: u64,
    expires_at: Instant,
    reset_unix_seconds: u64,
}

impl InMemoryPublicRateLimiter {
    /// Increments one fixed-window key and returns the new count plus reset epoch.
    pub fn increment(&self, key: &str, window: Duration) -> (u64, u64) {
        let now = Instant::now();
        let reset_unix_seconds = unix_now().saturating_add(window.as_secs());
        let mut inner = self.inner.lock().expect("public limiter mutex poisoned");
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

/// Checks public API rate limits before expensive DB/cache work.
pub async fn check_public_rate_limit(
    state: &AppState,
    endpoint_class: &'static str,
    headers: &HeaderMap,
    connect_info: Option<&ConnectInfo<SocketAddr>>,
) -> RateLimitDecision {
    let config = &state.config().public_api;
    let limit = config.rate_limit_max_requests_per_ip;
    let window = Duration::from_secs(config.rate_limit_window_seconds);
    let reset_unix_seconds = current_window_reset(config.rate_limit_window_seconds);

    if !config.rate_limit_enabled {
        return RateLimitDecision {
            allowed: true,
            limit,
            remaining: limit,
            retry_after_seconds: 0,
            reset_unix_seconds,
        };
    }

    let client_key = client_key(headers, connect_info, config);
    let window_start = unix_now() / config.rate_limit_window_seconds;
    let key = format!("public-api:{endpoint_class}:{client_key}:{window_start}");
    let count = match state.cache().increment_with_ttl(&key, window).await {
        Some(count) => count,
        None => {
            let (count, _) = state.public_rate_limiter().increment(&key, window);
            count
        }
    };
    let allowed = count <= limit;
    let remaining = limit.saturating_sub(count);
    RateLimitDecision {
        allowed,
        limit,
        remaining,
        retry_after_seconds: retry_after(reset_unix_seconds),
        reset_unix_seconds,
    }
}

fn client_key(
    headers: &HeaderMap,
    connect_info: Option<&ConnectInfo<SocketAddr>>,
    config: &PublicApiConfig,
) -> String {
    let direct_ip = connect_info
        .map(|ConnectInfo(addr)| addr.ip())
        .unwrap_or(IpAddr::from([0, 0, 0, 0]));

    if config.trust_proxy_headers && is_trusted_proxy(direct_ip, &config.trusted_proxy_cidrs) {
        if let Some(forwarded) = first_forwarded_for(headers) {
            return forwarded.to_string();
        }
    }

    direct_ip.to_string()
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
