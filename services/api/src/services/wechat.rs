//! 微信 code2session 客户端模块，封装小程序登录凭证换取 openid/session_key 的外部调用。

use std::{
    error::Error,
    fmt,
    io::Write,
    process::{Command, Stdio},
};

use serde::Deserialize;

const CODE2SESSION_URL: &str = "https://api.weixin.qq.com/sns/jscode2session";

/// WechatCodeSession 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WechatCodeSession {
    pub openid: String,
    pub unionid: Option<String>,
}

/// 微信 code2session 客户端抽象，便于测试中替换真实 HTTP 调用。
pub trait WechatCodeSessionClient: Send + Sync {
    /// 调用微信 code2session 接口，将小程序临时 code 换取可信 openid。
    fn code2session(
        &self,
        app_id: &str,
        app_secret: &str,
        code: &str,
    ) -> anyhow::Result<WechatCodeSession>;
}

/// CurlWechatCodeSessionClient 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Clone, Default)]
pub struct CurlWechatCodeSessionClient;

/// 微信 code2session 失败类型，区分缺少配置、网络错误和微信业务错误。
#[derive(Debug)]
pub enum WechatCodeSessionError {
    Rejected { code: i64, message: String },
    MissingOpenid,
    CurlUnavailable(std::io::Error),
    CurlIo(std::io::Error),
    CurlFailed(String),
    InvalidResponse(serde_json::Error),
}

impl fmt::Display for WechatCodeSessionError {
    /// 执行 `fmt` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rejected { code, message } => {
                write!(formatter, "wechat rejected login code: {message} ({code})")
            }
            Self::MissingOpenid => write!(
                formatter,
                "wechat code2session response did not include openid"
            ),
            Self::CurlUnavailable(error) => write!(
                formatter,
                "failed to run curl for wechat code2session: {error}"
            ),
            Self::CurlIo(error) => write!(
                formatter,
                "failed to communicate with curl for wechat code2session: {error}"
            ),
            Self::CurlFailed(message) => {
                write!(formatter, "wechat code2session request failed: {message}")
            }
            Self::InvalidResponse(error) => write!(
                formatter,
                "failed to parse wechat code2session response: {error}"
            ),
        }
    }
}

impl Error for WechatCodeSessionError {
    /// 执行 `source` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CurlUnavailable(error) => Some(error),
            Self::CurlIo(error) => Some(error),
            Self::InvalidResponse(error) => Some(error),
            _ => None,
        }
    }
}

/// WechatCodeSessionResponse 数据结构，定义当前模块对外暴露或内部复用的稳定数据边界。
#[derive(Debug, Deserialize)]
struct WechatCodeSessionResponse {
    openid: Option<String>,
    unionid: Option<String>,
    errcode: Option<i64>,
    errmsg: Option<String>,
}

/// 执行 `build curl config` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn build_curl_config(app_id: &str, app_secret: &str, code: &str) -> String {
    [
        "silent".to_owned(),
        "show-error".to_owned(),
        "fail".to_owned(),
        "get".to_owned(),
        "connect-timeout = 5".to_owned(),
        "max-time = 10".to_owned(),
        format!("url = {}", curl_config_quote(CODE2SESSION_URL)),
        format!(
            "data-urlencode = {}",
            curl_config_quote(&format!("appid={app_id}"))
        ),
        format!(
            "data-urlencode = {}",
            curl_config_quote(&format!("secret={app_secret}"))
        ),
        format!(
            "data-urlencode = {}",
            curl_config_quote(&format!("js_code={code}"))
        ),
        format!(
            "data-urlencode = {}",
            curl_config_quote("grant_type=authorization_code")
        ),
    ]
    .join("\n")
}

/// 执行 `curl config quote` 对应的服务端逻辑，并保持当前模块的输入校验、错误传播和状态不变量。
fn curl_config_quote(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\r', '\n'], "");
    format!("\"{}\"", escaped)
}

impl WechatCodeSessionClient for CurlWechatCodeSessionClient {
    /// 调用微信 code2session 接口，将小程序临时 code 换取可信 openid。
    fn code2session(
        &self,
        app_id: &str,
        app_secret: &str,
        code: &str,
    ) -> anyhow::Result<WechatCodeSession> {
        let curl_config = build_curl_config(app_id, app_secret, code);
        let mut child = Command::new("curl")
            .args(["--config", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(WechatCodeSessionError::CurlUnavailable)?;

        child
            .stdin
            .as_mut()
            .expect("curl stdin must be piped")
            .write_all(curl_config.as_bytes())
            .map_err(WechatCodeSessionError::CurlIo)?;

        let output = child
            .wait_with_output()
            .map_err(WechatCodeSessionError::CurlUnavailable)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(WechatCodeSessionError::CurlFailed(if stderr.is_empty() {
                format!("curl exited with status {}", output.status)
            } else {
                stderr
            })
            .into());
        }

        let response: WechatCodeSessionResponse = serde_json::from_slice(&output.stdout)
            .map_err(WechatCodeSessionError::InvalidResponse)?;

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
