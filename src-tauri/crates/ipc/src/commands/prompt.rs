use prompt_core::{AppError, CreatePromptInput, Prompt, PromptQuery, UpdatePromptInput};
use tauri::State;

use crate::state::AppState;

// Phases 0 冻结的命令签名（对应 M1 分工清单 §3、架构方案 §2.2.2）。
// 命令名 + 入参/出参类型即前后端契约，变更需广播。
// M1-B1 将此处 `todo!()` 替换为对 `state.prompt_repo` 的 trait 调用。

/// 获取提示词列表（支持 keyword / category / is_pinned 筛选 + limit / offset 分页）。
#[tauri::command]
pub fn get_prompts(
    state: State<'_, AppState>,
    query: PromptQuery,
) -> Result<Vec<Prompt>, AppError> {
    let _ = (state, query);
    todo!("M1-B1 实现：state.prompt_repo.find_all(&query)")
}

/// 创建提示词。
#[tauri::command]
pub fn create_prompt(
    state: State<'_, AppState>,
    input: CreatePromptInput,
) -> Result<Prompt, AppError> {
    let _ = (state, input);
    todo!("M1-B1 实现：state.prompt_repo.create(&input)")
}

/// 更新提示词（字段级局部更新）。
#[tauri::command]
pub fn update_prompt(
    state: State<'_, AppState>,
    id: String,
    input: UpdatePromptInput,
) -> Result<Prompt, AppError> {
    let _ = (state, id, input);
    todo!("M1-B1 实现：state.prompt_repo.update(&id, &input)")
}

/// 删除提示词（软删除，标记 deleted_at）。
#[tauri::command]
pub fn delete_prompt(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let _ = (state, id);
    todo!("M1-B1 实现：state.prompt_repo.delete(&id)")
}

/// 全文搜索提示词（FTS5）。
#[tauri::command]
pub fn search_prompts(
    state: State<'_, AppState>,
    keyword: String,
) -> Result<Vec<Prompt>, AppError> {
    let _ = (state, keyword);
    todo!("M1-B1 实现：state.prompt_repo.search(&keyword)")
}