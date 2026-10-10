//! AI 提供商配置持久化（非敏感项：baseUrl/model/temperature/maxTokens/timeout）。
//!
//! 与 `storage.json` 一致，配置文件固定落在应用数据目录（不随数据目录一起移动）。
//! API Key 不入本文件，走 OS 钥匙串（见 `crates/crypto`），前端仅感知「是否已配置」布尔。

use std::fs;
use std::path::Path;

use prompt_core::{AiConfig, AppError};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crypto::{KeychainStore, SecretStore, AI_API_KEY_USER};

/// 配置文件固定名（位于应用数据目录）。
const CONFIG_FILE: &str = "ai_config.json";

/// 前端可见的 AI 配置：非敏感字段 + 「是否已配置 Key」布尔（绝不含 Key 明文）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfigView {
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_secs: u64,
    pub api_key_required: bool,
    pub desensitize: bool,
    pub sensitive_terms: Vec<String>,
    pub has_api_key: bool,
}

impl From<AiConfig> for AiConfigView {
    fn from(c: AiConfig) -> Self {
        Self {
            provider: c.provider,
            base_url: c.base_url,
            model: c.model,
            temperature: c.temperature,
            max_tokens: c.max_tokens,
            timeout_secs: c.timeout_secs,
            api_key_required: c.api_key_required,
            desensitize: c.desensitize,
            sensitive_terms: c.sensitive_terms,
            has_api_key: false,
        }
    }
}

/// 读取配置；配置缺失/损坏时回退默认值。
pub fn load_config(base: &Path) -> AiConfig {
    match fs::read_to_string(base.join(CONFIG_FILE)) {
        Ok(text) => serde_json::from_str::<AiConfig>(&text).unwrap_or_default(),
        Err(_) => AiConfig::default(),
    }
}

fn save_config(base: &Path, config: &AiConfig) -> Result<(), AppError> {
    let text = serde_json::to_string_pretty(config).map_err(|e| AppError::Io(e.to_string()))?;
    fs::write(base.join(CONFIG_FILE), text).map_err(|e| AppError::Io(e.to_string()))
}

/// 是否已在钥匙串中配置了 AI API Key（不读取 Key 内容）。
fn has_api_key() -> bool {
    KeychainStore::with_user(AI_API_KEY_USER)
        .and_then(|s| s.get())
        .map(|o| o.is_some())
        .unwrap_or(false)
}

fn app_dir_err(e: tauri::Error) -> AppError {
    AppError::Io(e.to_string())
}

/// 读取当前 AI 配置（含 `hasApiKey`，不含 Key 明文）。
#[tauri::command]
pub fn get_ai_config(app: AppHandle) -> Result<AiConfigView, AppError> {
    let base = app.path().app_data_dir().map_err(app_dir_err)?;
    let config = load_config(&base);
    Ok(AiConfigView {
        has_api_key: has_api_key(),
        ..AiConfigView::from(config)
    })
}

/// 保存 AI 配置（只写非敏感字段，Key 由 `set_ai_api_key` 单独管理）。
#[tauri::command]
pub fn set_ai_config(app: AppHandle, config: AiConfig) -> Result<AiConfigView, AppError> {
    let base = app.path().app_data_dir().map_err(app_dir_err)?;
    save_config(&base, &config)?;
    Ok(AiConfigView {
        has_api_key: has_api_key(),
        ..AiConfigView::from(config)
    })
}
