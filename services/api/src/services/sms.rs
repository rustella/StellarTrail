//! Aliyun SMS verification client and local test implementation.
//!
//! The production path calls the Dypnsapi RPC OpenAPI directly so the API can
//! ask Aliyun to generate, send, and later verify SMS codes. Local tests use an
//! in-memory implementation that returns `debug_code` without persisting
//! plaintext codes in the database.

use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fmt,
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex, Once},
    time::Duration,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use hmac::{Hmac, Mac};
use rand::Rng;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use rustls_pki_types::ServerName;
use serde::Deserialize;
use sha1::Sha1;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::config::SmsConfig;

const ALIYUN_DYPNSAPI_VERSION: &str = "2017-05-25";
const HTTPS_TIMEOUT: Duration = Duration::from_secs(10);
static RUSTLS_CRYPTO_PROVIDER: Once = Once::new();

type HmacSha1 = Hmac<Sha1>;

/// Request data needed to ask the configured SMS provider to send one code.
#[derive(Clone, Debug)]
pub struct SmsSendCodeRequest {
    pub phone: String,
    pub out_id: String,
    pub template_code: String,
    pub template_param: String,
}

/// Provider response after a code send request.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SmsSendCodeOutcome {
    pub request_id: Option<String>,
    pub debug_code: Option<String>,
}

/// Request data needed to verify a user-entered SMS code.
#[derive(Clone, Debug)]
pub struct SmsCheckCodeRequest {
    pub phone: String,
    pub out_id: String,
    pub verify_code: String,
}

/// Provider response after checking a user-entered SMS code.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SmsCheckCodeOutcome {
    pub request_id: Option<String>,
    pub passed: bool,
}

/// SMS verification client abstraction used by auth services and tests.
pub trait SmsVerificationClient: Send + Sync {
    /// Sends a verification code through the configured SMS provider.
    fn send_verify_code(
        &self,
        request: SmsSendCodeRequest,
    ) -> Result<SmsSendCodeOutcome, SmsVerificationError>;

    /// Checks a user-entered verification code against the configured SMS provider.
    fn check_verify_code(
        &self,
        request: SmsCheckCodeRequest,
    ) -> Result<SmsCheckCodeOutcome, SmsVerificationError>;
}

/// Aliyun Dypnsapi-backed SMS verification client.
#[derive(Clone, Debug)]
pub struct AliyunSmsVerificationClient {
    config: SmsConfig,
}

impl AliyunSmsVerificationClient {
    /// Creates a production SMS client from already-validated configuration.
    pub fn from_config(config: &SmsConfig) -> anyhow::Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }
}

impl SmsVerificationClient for AliyunSmsVerificationClient {
    fn send_verify_code(
        &self,
        request: SmsSendCodeRequest,
    ) -> Result<SmsSendCodeOutcome, SmsVerificationError> {
        let mut params = signed_common_params(&self.config);
        params.insert("Action".to_owned(), "SendSmsVerifyCode".to_owned());
        params.insert("PhoneNumber".to_owned(), request.phone);
        params.insert("SignName".to_owned(), self.config.sign_name.clone());
        params.insert("TemplateCode".to_owned(), request.template_code);
        insert_scheme_name(&mut params, &self.config);
        params.insert("TemplateParam".to_owned(), request.template_param);
        params.insert("CountryCode".to_owned(), "86".to_owned());
        params.insert("OutId".to_owned(), request.out_id);
        params.insert(
            "Interval".to_owned(),
            self.config.interval_seconds.to_string(),
        );
        params.insert(
            "ValidTime".to_owned(),
            self.config.valid_time_seconds.to_string(),
        );
        params.insert("ReturnVerifyCode".to_owned(), "false".to_owned());
        params.insert("CodeType".to_owned(), "1".to_owned());
        params.insert("DuplicateCheck".to_owned(), "false".to_owned());
        params.insert("EarlyWarn".to_owned(), "false".to_owned());

        let body = signed_rpc_get(&self.config, params)?;
        let response: SendSmsVerifyCodeResponse =
            serde_json::from_slice(&body).map_err(SmsVerificationError::InvalidResponse)?;
        ensure_success(
            response.code.as_deref(),
            response.success,
            response.message.as_deref(),
        )?;
        Ok(SmsSendCodeOutcome {
            request_id: response.request_id.or_else(|| {
                response
                    .model
                    .as_ref()
                    .and_then(|model| model.request_id.clone())
            }),
            debug_code: response.model.and_then(|model| model.verify_code),
        })
    }

    fn check_verify_code(
        &self,
        request: SmsCheckCodeRequest,
    ) -> Result<SmsCheckCodeOutcome, SmsVerificationError> {
        let mut params = signed_common_params(&self.config);
        params.insert("Action".to_owned(), "CheckSmsVerifyCode".to_owned());
        params.insert("CountryCode".to_owned(), "86".to_owned());
        params.insert("OutId".to_owned(), request.out_id);
        params.insert("PhoneNumber".to_owned(), request.phone);
        insert_scheme_name(&mut params, &self.config);
        params.insert("VerifyCode".to_owned(), request.verify_code);

        let body = signed_rpc_get(&self.config, params)?;
        let response: CheckSmsVerifyCodeResponse =
            serde_json::from_slice(&body).map_err(SmsVerificationError::InvalidResponse)?;
        ensure_success(
            response.code.as_deref(),
            response.success,
            response.message.as_deref(),
        )?;
        let model = response.model.ok_or(SmsVerificationError::MissingModel)?;
        Ok(SmsCheckCodeOutcome {
            request_id: None,
            passed: model.verify_result.as_deref() == Some("PASS"),
        })
    }
}

/// In-memory SMS client used by local development and route tests.
#[derive(Clone, Default)]
pub struct InMemorySmsVerificationClient {
    codes: Arc<Mutex<HashMap<String, LocalSmsCode>>>,
}

#[derive(Clone, Debug)]
struct LocalSmsCode {
    phone: String,
    code: String,
}

impl SmsVerificationClient for InMemorySmsVerificationClient {
    fn send_verify_code(
        &self,
        request: SmsSendCodeRequest,
    ) -> Result<SmsSendCodeOutcome, SmsVerificationError> {
        let code = format!("{:06}", rand::thread_rng().gen_range(0..1_000_000));
        self.codes.lock().unwrap().insert(
            request.out_id.clone(),
            LocalSmsCode {
                phone: request.phone,
                code: code.clone(),
            },
        );
        Ok(SmsSendCodeOutcome {
            request_id: Some(format!("local-{}", Uuid::new_v4())),
            debug_code: Some(code),
        })
    }

    fn check_verify_code(
        &self,
        request: SmsCheckCodeRequest,
    ) -> Result<SmsCheckCodeOutcome, SmsVerificationError> {
        let matched = self
            .codes
            .lock()
            .unwrap()
            .get(&request.out_id)
            .is_some_and(|stored| {
                stored.phone == request.phone && stored.code == request.verify_code
            });
        Ok(SmsCheckCodeOutcome {
            request_id: Some(format!("local-{}", Uuid::new_v4())),
            passed: matched,
        })
    }
}

/// Aliyun SMS client error with enough detail to map provider failures to API errors.
#[derive(Debug)]
pub enum SmsVerificationError {
    Rejected { code: String, message: String },
    RateLimited { code: String, message: String },
    MissingModel,
    Network(std::io::Error),
    Tls(rustls::Error),
    InvalidHttpResponse(String),
    HttpStatus { status: u16, body: String },
    InvalidResponse(serde_json::Error),
}

impl fmt::Display for SmsVerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rejected { code, message } => {
                write!(formatter, "aliyun sms rejected request: {message} ({code})")
            }
            Self::RateLimited { code, message } => {
                write!(
                    formatter,
                    "aliyun sms rate limited request: {message} ({code})"
                )
            }
            Self::MissingModel => write!(formatter, "aliyun sms response did not include Model"),
            Self::Network(error) => write!(formatter, "aliyun sms network request failed: {error}"),
            Self::Tls(error) => write!(formatter, "aliyun sms tls handshake failed: {error}"),
            Self::InvalidHttpResponse(message) => {
                write!(formatter, "aliyun sms returned invalid HTTP: {message}")
            }
            Self::HttpStatus { status, body } => {
                write!(formatter, "aliyun sms returned HTTP {status}: {body}")
            }
            Self::InvalidResponse(error) => {
                write!(formatter, "failed to parse aliyun sms response: {error}")
            }
        }
    }
}

impl Error for SmsVerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Network(error) => Some(error),
            Self::Tls(error) => Some(error),
            Self::InvalidResponse(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct SendSmsVerifyCodeResponse {
    #[serde(rename = "Code")]
    code: Option<String>,
    #[serde(rename = "Message")]
    message: Option<String>,
    #[serde(rename = "Success")]
    success: Option<bool>,
    #[serde(rename = "RequestId")]
    request_id: Option<String>,
    #[serde(rename = "Model")]
    model: Option<SendSmsVerifyCodeModel>,
}

#[derive(Debug, Deserialize)]
struct SendSmsVerifyCodeModel {
    #[serde(rename = "RequestId")]
    request_id: Option<String>,
    #[serde(rename = "VerifyCode")]
    verify_code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CheckSmsVerifyCodeResponse {
    #[serde(rename = "Code")]
    code: Option<String>,
    #[serde(rename = "Message")]
    message: Option<String>,
    #[serde(rename = "Success")]
    success: Option<bool>,
    #[serde(rename = "Model")]
    model: Option<CheckSmsVerifyCodeModel>,
}

#[derive(Debug, Deserialize)]
struct CheckSmsVerifyCodeModel {
    #[serde(rename = "VerifyResult")]
    verify_result: Option<String>,
}

fn signed_common_params(config: &SmsConfig) -> BTreeMap<String, String> {
    let mut params = BTreeMap::new();
    params.insert("AccessKeyId".to_owned(), config.access_key_id.clone());
    params.insert("Format".to_owned(), "JSON".to_owned());
    params.insert("SignatureMethod".to_owned(), "HMAC-SHA1".to_owned());
    params.insert(
        "SignatureNonce".to_owned(),
        Uuid::new_v4().hyphenated().to_string(),
    );
    params.insert("SignatureVersion".to_owned(), "1.0".to_owned());
    params.insert(
        "Timestamp".to_owned(),
        aliyun_timestamp(OffsetDateTime::now_utc()),
    );
    params.insert("Version".to_owned(), ALIYUN_DYPNSAPI_VERSION.to_owned());
    params
}

fn insert_scheme_name(params: &mut BTreeMap<String, String>, config: &SmsConfig) {
    if !config.scheme_name.trim().is_empty() {
        params.insert("SchemeName".to_owned(), config.scheme_name.clone());
    }
}

fn signed_rpc_get(
    config: &SmsConfig,
    params: BTreeMap<String, String>,
) -> Result<Vec<u8>, SmsVerificationError> {
    let path = signed_rpc_path(&params, &config.access_key_secret)?;
    https_get(&config.endpoint, &path)
}

fn signed_rpc_path(
    params: &BTreeMap<String, String>,
    access_key_secret: &str,
) -> Result<String, SmsVerificationError> {
    let canonicalized = canonicalized_query(params);
    let string_to_sign = format!("GET&%2F&{}", encode_query_value(&canonicalized));
    let signing_key = format!("{access_key_secret}&");
    let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes()).map_err(|error| {
        SmsVerificationError::InvalidHttpResponse(format!("invalid HMAC key: {error}"))
    })?;
    mac.update(string_to_sign.as_bytes());
    let signature = STANDARD.encode(mac.finalize().into_bytes());

    let mut signed = params.clone();
    signed.insert("Signature".to_owned(), signature);
    Ok(format!("/?{}", canonicalized_query(&signed)))
}

fn canonicalized_query(params: &BTreeMap<String, String>) -> String {
    params
        .iter()
        .map(|(key, value)| format!("{}={}", encode_query_value(key), encode_query_value(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn ensure_success(
    code: Option<&str>,
    success: Option<bool>,
    message: Option<&str>,
) -> Result<(), SmsVerificationError> {
    if code == Some("OK") && success.unwrap_or(true) {
        return Ok(());
    }
    let code = code.unwrap_or("UNKNOWN").to_owned();
    let message = message.unwrap_or("unknown error").to_owned();
    if is_rate_limited_code(&code) {
        Err(SmsVerificationError::RateLimited { code, message })
    } else {
        Err(SmsVerificationError::Rejected { code, message })
    }
}

fn is_rate_limited_code(code: &str) -> bool {
    let code = code.to_ascii_lowercase();
    code.contains("throttl") || code.contains("frequency") || code.contains("limit")
}

fn https_get(host: &str, path: &str) -> Result<Vec<u8>, SmsVerificationError> {
    install_rustls_crypto_provider();
    let root_store = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let server_name = ServerName::try_from(host.to_owned()).map_err(|error| {
        SmsVerificationError::InvalidHttpResponse(format!("invalid host `{host}`: {error}"))
    })?;
    let socket = TcpStream::connect((host, 443)).map_err(SmsVerificationError::Network)?;
    socket
        .set_read_timeout(Some(HTTPS_TIMEOUT))
        .map_err(SmsVerificationError::Network)?;
    socket
        .set_write_timeout(Some(HTTPS_TIMEOUT))
        .map_err(SmsVerificationError::Network)?;
    let connection =
        ClientConnection::new(Arc::new(config), server_name).map_err(SmsVerificationError::Tls)?;
    let mut stream = StreamOwned::new(connection, socket);
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nAccept: application/json\r\nUser-Agent: StellarTrail/{}\r\nConnection: close\r\n\r\n",
        env!("CARGO_PKG_VERSION")
    );
    stream
        .write_all(request.as_bytes())
        .map_err(SmsVerificationError::Network)?;

    let mut response = Vec::new();
    match stream.read_to_end(&mut response) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof && !response.is_empty() => {
        }
        Err(error) => return Err(SmsVerificationError::Network(error)),
    }
    parse_http_response(&response)
}

fn install_rustls_crypto_provider() {
    RUSTLS_CRYPTO_PROVIDER.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

fn parse_http_response(response: &[u8]) -> Result<Vec<u8>, SmsVerificationError> {
    let header_end = find_bytes(response, b"\r\n\r\n").ok_or_else(|| {
        SmsVerificationError::InvalidHttpResponse("missing header terminator".to_owned())
    })?;
    let headers = std::str::from_utf8(&response[..header_end]).map_err(|error| {
        SmsVerificationError::InvalidHttpResponse(format!("headers are not UTF-8: {error}"))
    })?;
    let status = parse_status_code(headers)?;
    let mut body = response[(header_end + 4)..].to_vec();
    if header_contains(headers, "transfer-encoding", "chunked") {
        body = decode_chunked_body(&body)?;
    }
    if !(200..300).contains(&status) {
        return Err(SmsVerificationError::HttpStatus {
            status,
            body: String::from_utf8_lossy(&body).trim().to_owned(),
        });
    }
    Ok(body)
}

fn parse_status_code(headers: &str) -> Result<u16, SmsVerificationError> {
    let status_line = headers.lines().next().ok_or_else(|| {
        SmsVerificationError::InvalidHttpResponse("missing status line".to_owned())
    })?;
    status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| {
            SmsVerificationError::InvalidHttpResponse(format!(
                "missing status code in `{status_line}`"
            ))
        })?
        .parse::<u16>()
        .map_err(|error| {
            SmsVerificationError::InvalidHttpResponse(format!(
                "invalid status code in `{status_line}`: {error}"
            ))
        })
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

fn decode_chunked_body(body: &[u8]) -> Result<Vec<u8>, SmsVerificationError> {
    let mut index = 0;
    let mut decoded = Vec::new();
    loop {
        let line_end = find_bytes(&body[index..], b"\r\n").ok_or_else(|| {
            SmsVerificationError::InvalidHttpResponse("invalid chunk size line".to_owned())
        })? + index;
        let size_text = std::str::from_utf8(&body[index..line_end])
            .map_err(|error| {
                SmsVerificationError::InvalidHttpResponse(format!(
                    "chunk size is not UTF-8: {error}"
                ))
            })?
            .split(';')
            .next()
            .unwrap_or("")
            .trim();
        let size = usize::from_str_radix(size_text, 16).map_err(|error| {
            SmsVerificationError::InvalidHttpResponse(format!(
                "invalid chunk size `{size_text}`: {error}"
            ))
        })?;
        index = line_end + 2;
        if size == 0 {
            return Ok(decoded);
        }
        let chunk_end = index.checked_add(size).ok_or_else(|| {
            SmsVerificationError::InvalidHttpResponse("chunk size overflow".to_owned())
        })?;
        if chunk_end + 2 > body.len() || &body[chunk_end..(chunk_end + 2)] != b"\r\n" {
            return Err(SmsVerificationError::InvalidHttpResponse(
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

fn aliyun_timestamp(now: OffsetDateTime) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
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
    fn signed_rpc_path_url_encodes_signature_inputs() {
        let params = BTreeMap::from([
            ("AccessKeyId".to_owned(), "testid".to_owned()),
            ("Action".to_owned(), "SendSmsVerifyCode".to_owned()),
            ("Format".to_owned(), "JSON".to_owned()),
            ("PhoneNumber".to_owned(), "13800138000".to_owned()),
            ("SignatureMethod".to_owned(), "HMAC-SHA1".to_owned()),
            ("SignatureNonce".to_owned(), "nonce".to_owned()),
            ("SignatureVersion".to_owned(), "1.0".to_owned()),
            ("SignName".to_owned(), "example-sms-sign-name".to_owned()),
            ("Timestamp".to_owned(), "2026-06-02T00:00:00Z".to_owned()),
            ("Version".to_owned(), "2017-05-25".to_owned()),
        ]);

        let path = signed_rpc_path(&params, "secret").unwrap();

        assert!(path.starts_with("/?AccessKeyId=testid&Action=SendSmsVerifyCode"));
        assert!(path.contains("SignName=example-sms-sign-name"));
        assert!(path.contains("&Signature="));
    }

    #[test]
    fn local_client_returns_debug_code_and_checks_it() {
        let client = InMemorySmsVerificationClient::default();
        let send = client
            .send_verify_code(SmsSendCodeRequest {
                phone: "13800138000".to_owned(),
                out_id: "ticket-1".to_owned(),
                template_code: "100001".to_owned(),
                template_param: "{}".to_owned(),
            })
            .unwrap();
        let code = send.debug_code.unwrap();

        let ok = client
            .check_verify_code(SmsCheckCodeRequest {
                phone: "13800138000".to_owned(),
                out_id: "ticket-1".to_owned(),
                verify_code: code,
            })
            .unwrap();
        let wrong_phone = client
            .check_verify_code(SmsCheckCodeRequest {
                phone: "13900139000".to_owned(),
                out_id: "ticket-1".to_owned(),
                verify_code: "000000".to_owned(),
            })
            .unwrap();

        assert!(ok.passed);
        assert!(!wrong_phone.passed);
    }

    #[test]
    fn provider_success_requires_ok_code_and_success_flag() {
        assert!(ensure_success(Some("OK"), Some(true), None).is_ok());
        assert!(matches!(
            ensure_success(
                Some("isv.BUSINESS_LIMIT_CONTROL"),
                Some(false),
                Some("too often")
            ),
            Err(SmsVerificationError::RateLimited { .. })
        ));
        assert!(matches!(
            ensure_success(
                Some("InvalidAccessKeyId.NotFound"),
                Some(false),
                Some("bad key")
            ),
            Err(SmsVerificationError::Rejected { .. })
        ));
    }

    #[test]
    fn check_response_verify_result_must_pass() {
        let passed: CheckSmsVerifyCodeResponse =
            serde_json::from_str(r#"{"Code":"OK","Success":true,"Model":{"VerifyResult":"PASS"}}"#)
                .unwrap();
        let failed: CheckSmsVerifyCodeResponse = serde_json::from_str(
            r#"{"Code":"OK","Success":true,"Model":{"VerifyResult":"UNKNOWN"}}"#,
        )
        .unwrap();

        assert_eq!(passed.model.unwrap().verify_result.as_deref(), Some("PASS"));
        assert_ne!(failed.model.unwrap().verify_result.as_deref(), Some("PASS"));
    }

    #[test]
    fn check_request_omits_optional_case_auth_policy() {
        let config = SmsConfig {
            enabled: true,
            endpoint: "dypnsapi.aliyuncs.com".to_owned(),
            access_key_id: "access-key".to_owned(),
            access_key_secret: "secret".to_owned(),
            sign_name: "example-sms-sign-name".to_owned(),
            scheme_name: String::new(),
            valid_time_seconds: 300,
            interval_seconds: 60,
            login_register_template_code: "100001".to_owned(),
            change_bound_phone_template_code: "100002".to_owned(),
            password_reset_template_code: "100003".to_owned(),
            bind_new_phone_template_code: "100004".to_owned(),
            verify_bound_phone_template_code: "100005".to_owned(),
        };
        let mut params = signed_common_params(&config);
        params.insert("Action".to_owned(), "CheckSmsVerifyCode".to_owned());
        params.insert("CountryCode".to_owned(), "86".to_owned());
        params.insert("OutId".to_owned(), "ticket".to_owned());
        params.insert("PhoneNumber".to_owned(), "15696331949".to_owned());
        insert_scheme_name(&mut params, &config);
        params.insert("VerifyCode".to_owned(), "9045".to_owned());

        assert!(!params.contains_key("CaseAuthPolicy"));
        assert!(params.contains_key("VerifyCode"));
    }
}
