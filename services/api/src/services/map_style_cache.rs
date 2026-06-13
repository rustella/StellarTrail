//! Hosted map style JSON cache that periodically mirrors public MapTiler styles.

use std::{
    collections::HashMap,
    error::Error,
    fmt,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::{Arc, Once, RwLock},
    time::Duration,
};

use anyhow::Context;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use rustls_pki_types::ServerName;
use serde_json::Value;
use tracing::{info, warn};
use url::Url;

use crate::config::{MapConfig, MapStyleConfig};

const MAP_STYLE_REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 60);
const MAP_STYLE_REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
static RUSTLS_CRYPTO_PROVIDER: Once = Once::new();

/// In-memory cache of hosted map style JSON documents keyed by style id.
#[derive(Clone, Default)]
pub struct MapStyleCache {
    inner: Arc<RwLock<HashMap<String, String>>>,
}

impl MapStyleCache {
    /// Returns the cached style JSON body for one configured style id.
    pub fn style_json(&self, style_id: &str) -> Option<String> {
        self.inner
            .read()
            .expect("map style cache lock should not be poisoned")
            .get(style_id)
            .cloned()
    }

    /// Inserts a style JSON body directly for focused route tests.
    pub fn insert_for_tests(&self, style_id: impl Into<String>, body: impl Into<String>) {
        self.inner
            .write()
            .expect("map style cache lock should not be poisoned")
            .insert(style_id.into(), body.into());
    }

    /// Refreshes every configured upstream style once, preserving the previous cache on failures.
    pub async fn refresh_all(&self, config: &MapConfig) {
        let styles = config.styles.clone();
        let public_key = config.public_key.clone();
        let result =
            tokio::task::spawn_blocking(move || fetch_configured_styles(styles, public_key)).await;
        let results = match result {
            Ok(results) => results,
            Err(error) => {
                warn!(error = %error, "map style refresh worker failed to join");
                return;
            }
        };

        for result in results {
            match result {
                Ok((style_id, body)) => {
                    self.inner
                        .write()
                        .expect("map style cache lock should not be poisoned")
                        .insert(style_id.clone(), body);
                    info!(style_id, "refreshed hosted map style JSON");
                }
                Err(error) => warn!(error = %error, "failed to refresh hosted map style JSON"),
            }
        }
    }

    /// Starts the hourly background style refresh loop.
    pub fn start_refresh_worker(self, config: MapConfig) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(MAP_STYLE_REFRESH_INTERVAL).await;
                self.refresh_all(&config).await;
            }
        });
    }
}

fn fetch_configured_styles(
    styles: Vec<MapStyleConfig>,
    public_key: Option<String>,
) -> Vec<Result<(String, String), MapStyleFetchError>> {
    let Some(public_key) = public_key
        .map(|key| key.trim().to_owned())
        .filter(|key| !key.is_empty())
    else {
        return styles
            .into_iter()
            .map(|style| Err(MapStyleFetchError::MissingPublicKey { style_id: style.id }))
            .collect();
    };

    styles
        .into_iter()
        .map(|style| fetch_style(style, &public_key))
        .collect()
}

fn fetch_style(
    style: MapStyleConfig,
    public_key: &str,
) -> Result<(String, String), MapStyleFetchError> {
    let url =
        upstream_url_with_public_key(&style.upstream_style_url, public_key).map_err(|error| {
            MapStyleFetchError::InvalidUrl {
                style_id: style.id.clone(),
                message: error.to_string(),
            }
        })?;
    let body = https_get(&url).map_err(|error| MapStyleFetchError::Network {
        style_id: style.id.clone(),
        message: error.to_string(),
    })?;
    let _: Value =
        serde_json::from_slice(&body).map_err(|error| MapStyleFetchError::InvalidJson {
            style_id: style.id.clone(),
            message: error.to_string(),
        })?;
    let body = String::from_utf8(body).map_err(|error| MapStyleFetchError::InvalidJson {
        style_id: style.id.clone(),
        message: error.to_string(),
    })?;
    Ok((style.id, body))
}

fn upstream_url_with_public_key(style_url: &str, public_key: &str) -> anyhow::Result<Url> {
    let mut url = Url::parse(style_url).with_context(|| "map style URL must be absolute")?;
    let already_has_key = url.query_pairs().any(|(key, _)| key == "key");
    if !already_has_key {
        url.query_pairs_mut().append_pair("key", public_key);
    }
    Ok(url)
}

fn https_get(url: &Url) -> anyhow::Result<Vec<u8>> {
    install_rustls_crypto_provider();
    if url.scheme() != "https" {
        anyhow::bail!("map style upstream URL must use https");
    }
    let host = url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("map style upstream URL must include host"))?
        .to_owned();
    let port = url.port_or_known_default().unwrap_or(443);
    let mut path = url.path().to_owned();
    if path.is_empty() {
        path.push('/');
    }
    if let Some(query) = url.query() {
        path.push('?');
        path.push_str(query);
    }

    let root_store = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let server_name = ServerName::try_from(host.clone())
        .with_context(|| format!("invalid map style upstream host `{host}`"))?;
    let addr = (host.as_str(), port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("map style upstream host did not resolve"))?;
    let socket = TcpStream::connect_timeout(&addr, MAP_STYLE_REQUEST_TIMEOUT)?;
    socket.set_read_timeout(Some(MAP_STYLE_REQUEST_TIMEOUT))?;
    socket.set_write_timeout(Some(MAP_STYLE_REQUEST_TIMEOUT))?;
    let connection = ClientConnection::new(Arc::new(config), server_name)?;
    let mut stream = StreamOwned::new(connection, socket);
    let host_header = if port == 443 {
        host
    } else {
        format!("{host}:{port}")
    };
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {host_header}\r\nAccept: application/json\r\nUser-Agent: StellarTrail/{}\r\nConnection: close\r\n\r\n",
        env!("CARGO_PKG_VERSION")
    );
    stream.write_all(request.as_bytes())?;

    let mut response = Vec::new();
    match stream.read_to_end(&mut response) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof && !response.is_empty() => {
        }
        Err(error) => return Err(error.into()),
    }
    parse_http_response(&response)
}

fn install_rustls_crypto_provider() {
    RUSTLS_CRYPTO_PROVIDER.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

fn parse_http_response(response: &[u8]) -> anyhow::Result<Vec<u8>> {
    let header_end = find_bytes(response, b"\r\n\r\n")
        .ok_or_else(|| anyhow::anyhow!("missing header terminator"))?;
    let headers = std::str::from_utf8(&response[..header_end])
        .with_context(|| "map style response headers are not UTF-8")?;
    let status = parse_status_code(headers)?;
    let mut body = response[(header_end + 4)..].to_vec();
    if header_contains(headers, "transfer-encoding", "chunked") {
        body = decode_chunked_body(&body)?;
    }
    if !(200..300).contains(&status) {
        anyhow::bail!(
            "map style upstream returned HTTP {status}: {}",
            String::from_utf8_lossy(&body).trim()
        );
    }
    Ok(body)
}

fn parse_status_code(headers: &str) -> anyhow::Result<u16> {
    let status_line = headers
        .lines()
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing HTTP status line"))?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("missing HTTP status code"))?
        .parse::<u16>()?;
    Ok(status)
}

fn header_contains(headers: &str, name: &str, expected: &str) -> bool {
    headers.lines().skip(1).any(|line| {
        let Some((candidate, value)) = line.split_once(':') else {
            return false;
        };
        candidate.trim().eq_ignore_ascii_case(name)
            && value
                .split(',')
                .any(|part| part.trim().eq_ignore_ascii_case(expected))
    })
}

fn decode_chunked_body(body: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut index = 0;
    loop {
        let line_end = find_bytes(&body[index..], b"\r\n")
            .ok_or_else(|| anyhow::anyhow!("invalid chunked response"))?
            + index;
        let size_text = std::str::from_utf8(&body[index..line_end])
            .with_context(|| "chunk size is not UTF-8")?;
        let size_hex = size_text.split(';').next().unwrap_or_default().trim();
        let size = usize::from_str_radix(size_hex, 16)
            .with_context(|| format!("invalid chunk size `{size_hex}`"))?;
        index = line_end + 2;
        if size == 0 {
            break;
        }
        let chunk_end = index + size;
        if chunk_end + 2 > body.len() {
            anyhow::bail!("chunk exceeds response body");
        }
        output.extend_from_slice(&body[index..chunk_end]);
        index = chunk_end + 2;
    }
    Ok(output)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[derive(Debug)]
enum MapStyleFetchError {
    MissingPublicKey { style_id: String },
    InvalidUrl { style_id: String, message: String },
    Network { style_id: String, message: String },
    InvalidJson { style_id: String, message: String },
}

impl fmt::Display for MapStyleFetchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPublicKey { style_id } => {
                write!(
                    formatter,
                    "map style `{style_id}` cannot refresh without public key"
                )
            }
            Self::InvalidUrl { style_id, message } => {
                write!(
                    formatter,
                    "map style `{style_id}` has invalid upstream URL: {message}"
                )
            }
            Self::Network { style_id, message } => {
                write!(
                    formatter,
                    "map style `{style_id}` refresh failed: {message}"
                )
            }
            Self::InvalidJson { style_id, message } => {
                write!(
                    formatter,
                    "map style `{style_id}` returned invalid JSON: {message}"
                )
            }
        }
    }
}

impl Error for MapStyleFetchError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MapConfig;

    #[tokio::test]
    async fn refresh_failure_keeps_previous_cache() {
        let cache = MapStyleCache::default();
        cache.insert_for_tests("outdoor", r#"{"version":8,"sources":{},"layers":[]}"#);
        let config = MapConfig {
            public_key: None,
            ..MapConfig::default()
        };

        cache.refresh_all(&config).await;

        assert_eq!(
            cache.style_json("outdoor").as_deref(),
            Some(r#"{"version":8,"sources":{},"layers":[]}"#)
        );
    }
}
