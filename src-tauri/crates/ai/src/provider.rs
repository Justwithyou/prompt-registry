//! 模型提供商抽象与 OpenAI 兼容实现（对应《AI 辅助生成提示词设计方案》§5.1）。

use std::time::Duration;

use prompt_core::{AiConfig, AppError};

use crate::ai_err;

/// 模型调用抽象，后续「调优 / 本地模型」在同一 trait 上新增方法与实现。
pub trait Provider: Send + Sync {
    /// 发起一次对话补全，返回模型输出的原始文本（JSON 或纯文本兜底）。
    fn generate(
        &self,
        cfg: &AiConfig,
        api_key: &str,
        system: &str,
        user: &str,
    ) -> Result<String, AppError>;
}

/// OpenAI 兼容 HTTP 协议实现（`POST {base_url}/chat/completions`）。
///
/// 无内部状态，可安全共享；命令层按需构造。
pub struct OpenAiCompatibleProvider {
    client: reqwest::blocking::Client,
}

impl OpenAiCompatibleProvider {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Self { client }
    }
}

impl Default for OpenAiCompatibleProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for OpenAiCompatibleProvider {
    fn generate(
        &self,
        cfg: &AiConfig,
        api_key: &str,
        system: &str,
        user: &str,
    ) -> Result<String, AppError> {
        let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": cfg.model,
            "temperature": cfg.temperature,
            "max_tokens": cfg.max_tokens,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user },
            ],
        });

        let resp = self
            .client
            .post(&url)
            .timeout(Duration::from_secs(cfg.timeout_secs.max(1)))
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .map_err(|e| ai_err(format!("请求模型失败: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(ai_err(format!(
                "模型返回错误 HTTP {status}: {}",
                truncate(&text, 160)
            )));
        }

        let val: serde_json::Value = resp
            .json()
            .map_err(|e| ai_err(format!("解析模型响应失败: {e}")))?;
        let content = val
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ai_err("模型响应缺少 choices[0].message.content"))?;
        Ok(content.to_string())
    }
}

/// 截断错误正文，避免把超大响应体塞进错误信息（也不泄露敏感内容）。
fn truncate(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let head: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}