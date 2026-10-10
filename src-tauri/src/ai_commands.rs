//! AI 命令层：写入 API Key（OS 钥匙串）+ 生成 / 调优 / 提取变量（含脱敏管线）。

use prompt_core::{AiConfig, AppError, PromptDraft};
use tauri::{AppHandle, Manager};

use ai::{
    extract_variables_ai, generate_prompt, is_local, polish_prompt, sanitize, GenerateOptions,
    MaskedText, OpenAiCompatibleProvider, PolishOptions,
};
use crypto::{KeychainStore, SecretStore, AI_API_KEY_USER};

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

/// 读取配置 + API Key（本地模型允许留空 Key，云端模型缺失 Key 时报错）。
fn load_context(app: &AppHandle) -> Result<(AiConfig, String), AppError> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Io(e.to_string()))?;
    let config = ai_config::load_config(&base);
    let key = KeychainStore::with_user(AI_API_KEY_USER)?
        .get()?
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if key.is_empty() && config.api_key_required {
        return Err(AppError::Ai(
            "尚未配置 API Key，请到「设置 → AI 辅助」填写".into(),
        ));
    }
    Ok((config, key))
}

/// 出网前脱敏：仅当开启脱敏且指向远端时生效；返回（脱敏文本，可选映射）。
fn maybe_sanitize(config: &AiConfig, text: &str) -> (String, Option<MaskedText>) {
    if config.desensitize && !is_local(&config.base_url) {
        let masked = sanitize(text, &config.sensitive_terms);
        (masked.text.clone(), Some(masked))
    } else {
        (text.to_string(), None)
    }
}

/// 回网后还原草稿中的敏感占位。
fn restore_draft(mask: Option<&MaskedText>, draft: &mut PromptDraft) {
    let Some(m) = mask else { return };
    draft.title = m.restore(&draft.title);
    draft.content = m.restore(&draft.content);
    draft.description = m.restore(&draft.description);
    draft.suggested_tags = draft.suggested_tags.iter().map(|t| m.restore(t)).collect();
    if let Some(c) = draft.suggested_category.take() {
        draft.suggested_category = Some(m.restore(&c));
    }
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
    let (config, key) = load_context(&app)?;
    let (text, mask) = maybe_sanitize(&config, &requirement);
    let provider = OpenAiCompatibleProvider::new();
    let opts = options.unwrap_or_default();
    let mut draft = generate_prompt(&provider, &config, &key, &text, &opts)?;
    restore_draft(mask.as_ref(), &mut draft);
    Ok(draft)
}

/// 「AI 调优」：改写一条提示词正文，返回调优后草稿（不落库）。
#[tauri::command]
pub fn ai_polish_prompt(
    app: AppHandle,
    content: String,
    options: Option<PolishOptions>,
) -> Result<PromptDraft, AppError> {
    if content.trim().is_empty() {
        return Err(AppError::Ai("待调优的提示词正文不能为空".into()));
    }
    let (config, key) = load_context(&app)?;
    let (text, mask) = maybe_sanitize(&config, &content);
    let provider = OpenAiCompatibleProvider::new();
    let opts = options.unwrap_or_default();
    let mut draft = polish_prompt(&provider, &config, &key, &text, &opts)?;
    restore_draft(mask.as_ref(), &mut draft);
    Ok(draft)
}

/// 「AI 提取变量」：把正文参数化改写为含 `{{变量}}` 占位的版本（不落库）。
#[tauri::command]
pub fn ai_extract_variables(app: AppHandle, content: String) -> Result<PromptDraft, AppError> {
    if content.trim().is_empty() {
        return Err(AppError::Ai("待提取变量的提示词正文不能为空".into()));
    }
    let (config, key) = load_context(&app)?;
    let (text, mask) = maybe_sanitize(&config, &content);
    let provider = OpenAiCompatibleProvider::new();
    let mut draft = extract_variables_ai(&provider, &config, &key, &text)?;
    restore_draft(mask.as_ref(), &mut draft);
    Ok(draft)
}
