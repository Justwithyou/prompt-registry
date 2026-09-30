//! WebDAV 客户端（M4-C2）：`RemoteStore` 实现，覆盖 PUT/GET/PROPFIND/MKCOL。
//!
//! 认证采用 Basic Auth（用户名 + 令牌；令牌即 WebDAV 应用专用的密码）。
//! PROPFIND 返回的多播 XML 用轻量正则提取 `<href>`，并规整为相对路径。

use std::time::Duration;

use prompt_core::{AppError, RemoteStore, SyncConfig};
use regex::Regex;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

fn sync_err(msg: impl Into<String>) -> AppError {
    AppError::Sync(msg.into())
}

fn network_err(msg: impl Into<String>) -> AppError {
    AppError::Network(msg.into())
}

/// 基于 `reqwest`（阻塞式 + rustls）的 WebDAV 客户端。
#[derive(Debug, Clone)]
pub struct WebDavClient {
    base_url: String,
    username: String,
    token: Option<String>,
}

impl WebDavClient {
    /// 构造客户端；`base_url` 末尾斜杠会被规整掉。
    pub fn new(base_url: &str, username: &str, token: Option<&str>) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            username: username.to_string(),
            token: token.map(|t| t.to_string()),
        }
    }

    /// 从同步配置直接构造。
    pub fn from_config(config: &SyncConfig) -> Self {
        Self::new(&config.url, &config.username, config.token.as_deref())
    }

    fn client(&self) -> reqwest::blocking::Client {
        reqwest::blocking::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new())
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }

    fn auth(&self, rb: reqwest::blocking::RequestBuilder) -> reqwest::blocking::RequestBuilder {
        rb.basic_auth(&self.username, Some(self.token.as_deref().unwrap_or("")))
    }

    fn send(
        &self,
        rb: reqwest::blocking::RequestBuilder,
    ) -> Result<reqwest::blocking::Response, AppError> {
        self.auth(rb).send().map_err(|e| network_err(e.to_string()))
    }
}

impl RemoteStore for WebDavClient {
    fn put(&self, path: &str, data: &[u8]) -> Result<(), AppError> {
        let resp = self.send(self.client().put(self.url(path)).body(data.to_vec()))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(network_err(format!("PUT 失败: HTTP {}", resp.status())))
        }
    }

    fn get(&self, path: &str) -> Result<Vec<u8>, AppError> {
        let resp = self.send(self.client().get(self.url(path)))?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(AppError::NotFound(format!("远端文件不存在: {path}")));
        }
        if !resp.status().is_success() {
            return Err(network_err(format!("GET 失败: HTTP {}", resp.status())));
        }
        resp.bytes()
            .map(|b| b.to_vec())
            .map_err(|e| network_err(e.to_string()))
    }

    fn propfind(&self, path: &str) -> Result<Vec<String>, AppError> {
        let method = reqwest::Method::from_bytes(b"PROPFIND")
            .map_err(|e| sync_err(format!("构造 PROPFIND 方法失败: {e}")))?;
        let resp = self.send(
            self.client()
                .request(method, self.url(path))
                .header("Depth", "1"),
        )?;
        if !resp.status().is_success() {
            return Err(network_err(format!(
                "PROPFIND 失败: HTTP {}",
                resp.status()
            )));
        }
        let body = resp.text().map_err(|e| network_err(e.to_string()))?;
        Ok(extract_hrefs(&body))
    }

    fn mkcol(&self, path: &str) -> Result<(), AppError> {
        let method = reqwest::Method::from_bytes(b"MKCOL")
            .map_err(|e| sync_err(format!("构造 MKCOL 方法失败: {e}")))?;
        let resp = self.send(self.client().request(method, self.url(path)))?;
        // MKCOL 对已存在目录返回 405，视为成功（幂等）。
        if resp.status().is_success() || resp.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED {
            Ok(())
        } else {
            Err(network_err(format!("MKCOL 失败: HTTP {}", resp.status())))
        }
    }
}

/// 从 PROPFIND 返回的 XML 中提取 `<href>` 文本，并规整为相对路径。
///
/// 兼容常见 WebDAV 服务的命名空间写法（`<d:href>` / `<D:href>` / `<href>`），
/// 并做基础 XML 实体反转义与 URL 百分号解码。
pub fn extract_hrefs(xml: &str) -> Vec<String> {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r#"(?s)<(?:[A-Za-z_][\w.-]*:)?href[^>]*>(.*?)</(?:[A-Za-z_][\w.-]*:)?href>"#)
            .expect("href 提取正则为常量，必然合法")
    });

    re.captures_iter(xml)
        .filter_map(|caps| caps.get(1))
        .map(|m| normalize_href(m.as_str()))
        .filter(|s| !s.is_empty())
        .collect()
}

/// 规整单条 href：XML 反转义 → URL 解码 → 去掉末尾斜杠。
fn normalize_href(raw: &str) -> String {
    let unescaped = unescape_xml(raw.trim());
    let decoded = percent_decode(&unescaped);
    decoded.trim_end_matches('/').to_string()
}

/// 基础 XML 实体反转义。
fn unescape_xml(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// URL 百分号解码（`%XX` → 原始字节，按 UTF-8 重组）。
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
