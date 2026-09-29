use prompt_core::{
    AppError, BatchUpdateInput, CreatePromptInput, Prompt, PromptQuery, PromptRelationsInput,
    UpdatePromptInput,
};
use tauri::State;

use crate::state::AppState;

// Phase 0 冻结的命令签名（对应 M1 分工清单 §3、架构方案 §2.2.2）。
// 命令名 + 入参/出参类型即前后端契约，变更需广播。
// M1-B1 已完成：各命令从 `State<AppState>` 取 repository，仅调用 trait 方法，不触碰具体存储实现。

/// 获取提示词列表（支持 keyword / category / is_pinned 筛选 + limit / offset 分页）。
#[tauri::command]
pub fn get_prompts(
    state: State<'_, AppState>,
    query: PromptQuery,
) -> Result<Vec<Prompt>, AppError> {
    state.prompt_repo.find_all(&query)
}

/// 创建提示词。
#[tauri::command]
pub fn create_prompt(
    state: State<'_, AppState>,
    input: CreatePromptInput,
) -> Result<Prompt, AppError> {
    state.prompt_repo.create(&input)
}

/// 更新提示词（字段级局部更新）。
#[tauri::command]
pub fn update_prompt(
    state: State<'_, AppState>,
    id: String,
    input: UpdatePromptInput,
) -> Result<Prompt, AppError> {
    state.prompt_repo.update(&id, &input)
}

/// 删除提示词（软删除，标记 deleted_at）。
#[tauri::command]
pub fn delete_prompt(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.prompt_repo.delete(&id)
}

/// 全文搜索提示词（FTS5）。
#[tauri::command]
pub fn search_prompts(
    state: State<'_, AppState>,
    keyword: String,
) -> Result<Vec<Prompt>, AppError> {
    state.prompt_repo.search(&keyword)
}

/// 一次性更新某条提示词的组织关系与快捷标记（M2 组织层）。
#[tauri::command]
pub fn set_prompt_relations(
    state: State<'_, AppState>,
    id: String,
    relations: PromptRelationsInput,
) -> Result<Prompt, AppError> {
    state.prompt_repo.set_relations(&id, &relations)
}

/// 批量操作（批量打标签 / 移动项目 / 删除）。
#[tauri::command]
pub fn batch_update_prompts(
    state: State<'_, AppState>,
    ids: Vec<String>,
    batch: BatchUpdateInput,
) -> Result<(), AppError> {
    state.prompt_repo.batch_update(&ids, &batch)
}
