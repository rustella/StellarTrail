//! WeChat code2session client module wrapping the external call that exchanges Mini Program login credentials for openid/session_key.

use std::{
    error::Error,
    fmt,
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
    time::Duration,
};

use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use rustls_pki_types::ServerName;
use serde::Deserialize;

const WECHAT_API_HOST: &str = "api.weixin.qq.com";
const CODE2SESSION_PATH: &str = "/sns/jscode2session";
const CODE2SESSION_TIMEOUT: Duration = Duration::from_secs(10);

/// Stable data boundary for `WechatCodeSession`, exposed by or reused within this module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WechatCodeSession {
    pub openid: String,
    pub unionid: Option<String>,
}

/// WeChat code2session client abstraction that lets tests replace the real HTTP call.
pub trait WechatCodeSessionClient: Send + Sync {
    /// Calls the WeChat code2session API to exchange a temporary Mini Program code for a trusted openid.
    fn code2session(
        &self,
        app_id: &str,
        app_secret: &str,
        code: &str,
    ) -> anyhow::Result<WechatCodeSession>;
}

/// Stable data boundary for `HttpWechatCodeSessionClient`, exposed by or reused within this module.
#[derive(Clone, Default)]
pub struct HttpWechatCodeSessionClient;

/// WeChat code2session failure type that distinguishes missing configuration, network errors, and WeChat business errors.
#[derive(Debug)]
pub enum WechatCodeSessionError {
    Rejected { code: i64, message: String },
    MissingOpenid,
    Network(std::io::Error),
    Tls(rustls::Error),
    InvalidHttpResponse(String),
    HttpStatus { status: u16, body: String },
    InvalidResponse(serde_json::Error),
}

impl fmt::Display for WechatCodeSessionError {
    /// Runs the `fmt` server-side flow while preserving input validation, error propagation, and state invariants.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rejected { code, message } => {
                write!(formatter, "wechat rejected login code: {message} ({code})")
            }
            Self::MissingOpenid => write!(
                formatter,
                "wechat code2session response did not include openid"
            ),
            Self::Network(error) => write!(
                formatter,
                "wechat code2session network request failed: {error}"
            ),
            Self::Tls(error) => write!(
                formatter,
                "wechat code2session tls handshake failed: {error}"
            ),
            Self::InvalidHttpResponse(message) => {
                write!(
                    formatter,
                    "wechat code2session returned invalid HTTP: {message}"
                )
            }
            Self::HttpStatus { status, body } => {
                write!(
                    formatter,
                    "wechat code2session returned HTTP {status}: {body}"
                )
            }
            Self::InvalidResponse(error) => write!(
                formatter,
                "failed to parse wechat code2session response: {error}"
            ),
        }
    }
}

impl Error for WechatCodeSessionError {
    /// Runs the `source` server-side flow while preserving input validation, error propagation, and state invariants.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Network(error) => Some(error),
            Self::Tls(error) => Some(error),
            Self::InvalidResponse(error) => Some(error),
            _ => None,
        }
    }
}

/// Stable data boundary for `WechatCodeSessionResponse`, exposed by or reused within this module.
#[derive(Debug, Deserialize)]
struct WechatCodeSessionResponse {
    openid: Option<String>,
    unionid: Option<String>,
    errcode: Option<i64>,
    errmsg: Option<String>,
}

impl WechatCodeSessionClient for HttpWechatCodeSessionClient {
    /// Calls the WeChat code2session API to exchange a temporary Mini Program code for a trusted openid.
    fn code2session(
        &self,
        app_id: &str,
        app_secret: &str,
        code: &str,
    ) -> anyhow::Result<WechatCodeSession> {
        let path = build_code2session_path(app_id, app_secret, code);
        let body = https_get(WECHAT_API_HOST, &path)?;
        let response: WechatCodeSessionResponse =
            serde_json::from_slice(&body).map_err(WechatCodeSessionError::InvalidResponse)?;

        if let Some(errcode) = response.errcode.filter(|errcode| *errcode != 0) {
            return Err(WechatCodeSessionError::Rejected {
                code: errcode,
                message: response
                    .errmsg
                    .unwrap_or_else(|| "unknown error".to_owned()),
            }
            .into());
        }

        let openid = response
            .openid
            .map(|openid| openid.trim().to_owned())
            .filter(|openid| !openid.is_empty())
            .ok_or(WechatCodeSessionError::MissingOpenid)?;

        Ok(WechatCodeSession {
            openid,
            unionid: response.unionid,
        })
    }
}

fn build_code2session_path(app_id: &str, app_secret: &str, code: &str) -> String {
    format!(
        "{CODE2SESSION_PATH}?appid={}&secret={}&js_code={}&grant_type=authorization_code",
        encode_query_value(app_id),
        encode_query_value(app_secret),
        encode_query_value(code)
    )
}

fn https_get(host: &str, path: &str) -> Result<Vec<u8>, WechatCodeSessionError> {
    let root_store = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let server_name = ServerName::try_from(host.to_owned()).map_err(|error| {
        WechatCodeSessionError::InvalidHttpResponse(format!("invalid host `{host}`: {error}"))
    })?;
    let socket = TcpStream::connect((host, 443)).map_err(WechatCodeSessionError::Network)?;
    socket
        .set_read_timeout(Some(CODE2SESSION_TIMEOUT))
        .map_err(WechatCodeSessionError::Network)?;
    socket
        .set_write_timeout(Some(CODE2SESSION_TIMEOUT))
        .map_err(WechatCodeSessionError::Network)?;
    let connection = ClientConnection::new(Arc::new(config), server_name)
        .map_err(WechatCodeSessionError::Tls)?;
    let mut stream = StreamOwned::new(connection, socket);
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nAccept: application/json\r\nUser-Agent: StellarTrail/{}\r\nConnection: close\r\n\r\n",
        env!("CARGO_PKG_VERSION")
    );
    stream
        .write_all(request.as_bytes())
        .map_err(WechatCodeSessionError::Network)?;

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(WechatCodeSessionError::Network)?;
    parse_http_response(&response)
}

fn parse_http_response(response: &[u8]) -> Result<Vec<u8>, WechatCodeSessionError> {
    let header_end = find_bytes(response, b"\r\n\r\n").ok_or_else(|| {
        WechatCodeSessionError::InvalidHttpResponse("missing header terminator".to_owned())
    })?;
    let headers = std::str::from_utf8(&response[..header_end]).map_err(|error| {
        WechatCodeSessionError::InvalidHttpResponse(format!("headers are not UTF-8: {error}"))
    })?;
    let status = parse_status_code(headers)?;
    let mut body = response[(header_end + 4)..].to_vec();
    if header_contains(headers, "transfer-encoding", "chunked") {
        body = decode_chunked_body(&body)?;
    }
    if !(200..300).contains(&status) {
        return Err(WechatCodeSessionError::HttpStatus {
            status,
            body: String::from_utf8_lossy(&body).trim().to_owned(),
        });
    }
    Ok(body)
}

fn parse_status_code(headers: &str) -> Result<u16, WechatCodeSessionError> {
    let status_line = headers.lines().next().ok_or_else(|| {
        WechatCodeSessionError::InvalidHttpResponse("missing status line".to_owned())
    })?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| {
            WechatCodeSessionError::InvalidHttpResponse(format!(
                "missing status code in `{status_line}`"
            ))
        })?
        .parse::<u16>()
        .map_err(|error| {
            WechatCodeSessionError::InvalidHttpResponse(format!(
                "invalid status code in `{status_line}`: {error}"
            ))
        })?;
    Ok(status)
}

fn header_contains(headers: &str, name: &str, needle: &str) -> bool {
    headers.lines().skip(1).any(|line| {
        line.split_once(':')
            .map(|(header_name, value)| {
                header_name.trim().eq_ignore_ascii_case(name)
                    && value.to_ascii_lowercase().contains(needle)
            })
            .unwrap_or(false)
    })
}

fn decode_chunked_body(body: &[u8]) -> Result<Vec<u8>, WechatCodeSessionError> {
    let mut index = 0;
    let mut decoded = Vec::new();
    loop {
        let line_end = find_bytes(&body[index..], b"\r\n").ok_or_else(|| {
            WechatCodeSessionError::InvalidHttpResponse("invalid chunk size line".to_owned())
        })? + index;
        let size_text = std::str::from_utf8(&body[index..line_end])
            .map_err(|error| {
                WechatCodeSessionError::InvalidHttpResponse(format!(
                    "chunk size is not UTF-8: {error}"
                ))
            })?
            .split(';')
            .next()
            .unwrap_or("")
            .trim();
        let size = usize::from_str_radix(size_text, 16).map_err(|error| {
            WechatCodeSessionError::InvalidHttpResponse(format!(
                "invalid chunk size `{size_text}`: {error}"
            ))
        })?;
        index = line_end + 2;
        if size == 0 {
            return Ok(decoded);
        }
        let chunk_end = index.checked_add(size).ok_or_else(|| {
            WechatCodeSessionError::InvalidHttpResponse("chunk size overflow".to_owned())
        })?;
        if chunk_end + 2 > body.len() || &body[chunk_end..(chunk_end + 2)] != b"\r\n" {
            return Err(WechatCodeSessionError::InvalidHttpResponse(
                "chunk body is truncated".to_owned(),
            ));
        }
        decoded.extend_from_slice(&body[index..chunk_end]);
        index = chunk_end + 2;
    }
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn encode_query_value(value: &str) -> String {
    let mut encoded = String::new();
    value.bytes().for_each(|byte| {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    });
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code2session_path_url_encodes_credentials() {
        let path = build_code2session_path("wx id", "sec/ret+", "code?=1");

        assert_eq!(
            path,
            "/sns/jscode2session?appid=wx%20id&secret=sec%2Fret%2B&js_code=code%3F%3D1&grant_type=authorization_code"
        );
    }

    #[test]
    fn parses_chunked_http_response_body() {
        let response =
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n8\r\n{\"ok\":1}\r\n0\r\n\r\n";

        let body = parse_http_response(response).unwrap();

        assert_eq!(body, br#"{"ok":1}"#);
    }
}
