use prompt_core::{AppError, Prompt, PromptVersion};
use tauri::State;

use crate::state::AppState;

// Phase 0 冻结的命令签名（对应 M3 分工清单 §3.2）。
// 命令名 + 入参/出参类型即前后端契约，变更需广播。
// M3-B1：从 `State<AppState>` 取 version_repo，调用 `VersionRepository` trait 方法。

/// 列出某提示词的全部历史版本（按创建时间降序）。
#[tauri::command]
pub fn list_prompt_versions(
    state: State<'_, AppState>,
    prompt_id: String,
) -> Result<Vec<PromptVersion>, AppError> {
    state.version_repo.list(&prompt_id)
}

/// 获取单个版本详情。
#[tauri::command]
pub fn get_prompt_version(
    state: State<'_, AppState>,
    version_id: String,
) -> Result<PromptVersion, AppError> {
    state.version_repo.get(&version_id)
}

/// 回滚：从指定历史版本生成一条新版本（追加式，不覆盖历史）。
#[tauri::command]
pub fn rollback_prompt_version(
    state: State<'_, AppState>,
    prompt_id: String,
    version_id: String,
) -> Result<Prompt, AppError> {
    let prompt = state.version_repo.rollback(&prompt_id, &version_id)?;
    // 正文已变 → 记录 prompt 更新。
    crate::sync_hook::record_json(&state, "prompt", &prompt.id, "update", &prompt);
    // 回滚会追加一条新历史版本 → 记录到同步日志（追加式，跨设备可传播）。
    if let Ok(versions) = state.version_repo.list(&prompt_id) {
        if let Some(latest) = versions.first() {
            crate::sync_hook::record_json(&state, "version", &latest.id, "create", latest);
        }
    }
    Ok(prompt)
}
