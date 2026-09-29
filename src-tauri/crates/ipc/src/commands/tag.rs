use prompt_core::{AppError, CreateTagInput, Tag, UpdateTagInput};
use tauri::State;

use crate::state::AppState;

// Phase 0 冻结的命令签名（对应 M2 分工清单 §3.2）。
// 命令名 + 入参/出参类型即前后端契约，变更需广播。
// M2-B1：各命令从 `State<AppState>` 取 tag_repo，仅调用 trait 方法，不触碰具体存储实现。

/// 获取全部标签。
#[tauri::command]
pub fn list_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, AppError> {
    state.tag_repo.find_all()
}

/// 创建标签。
#[tauri::command]
pub fn create_tag(state: State<'_, AppState>, input: CreateTagInput) -> Result<Tag, AppError> {
    state.tag_repo.create(&input)
}

/// 更新标签（字段级局部更新）。
#[tauri::command]
pub fn update_tag(
    state: State<'_, AppState>,
    id: String,
    input: UpdateTagInput,
) -> Result<Tag, AppError> {
    state.tag_repo.update(&id, &input)
}

/// 删除标签。
#[tauri::command]
pub fn delete_tag(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.tag_repo.delete(&id)
}
