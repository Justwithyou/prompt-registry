//! AI 命令层：写入 API Key（OS 钥匙串）+ 「AI 生成提示词」。

use prompt_core::{AppError, PromptDraft};
use tauri::{AppHandle, Manager};

use ai::{generate_prompt, GenerateOptions, OpenAiCompatibleProvider};
use crypto::{AI_API_KEY_USER, KeychainStore, SecretStore};

use crate::ai_config;

/// 写入（或清空）AI API Key：只进 OS 钥匙串，永不落盘、回显或进日志。
#[tauri::command]
pub fn set_ai_api_key(api_key: String) -> Result<(), AppError> {
    let store = KeychainStore::with_user(AI_API_KEY_USER)?;
    if api_key.trim().is_empty() {
        store.delete()?;
    } else {
        store.set(api_key.as_bytes())?;
    }
    Ok(())
}

/// 「AI 生成提示词」：读配置 + API Key，单次调用模型，返回结构化草稿（不落库）。
#[tauri::command]
pub fn ai_generate_prompt(
    app: AppHandle,
    requirement: String,
    options: Option<GenerateOptions>,
) -> Result<PromptDraft, AppError> {
    if requirement.trim().is_empty() {
        return Err(AppError::Ai("需求描述不能为空".into()));
    }

    let base = app.path().app_data_dir().map_err(|e| AppError::Io(e.to_string()))?;
    let config = ai_config::load_config(&base);

    let key = KeychainStore::with_user(AI_API_KEY_USER)?
        .get()?
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Ai("尚未配置 API Key，请到「设置 → AI 辅助」填写".into()))?;

    let provider = OpenAiCompatibleProvider::new();
    let opts = options.unwrap_or_default();
    generate_prompt(&provider, &config, &key, &requirement, &opts)
}