use prompt_core::{AppError, Category, CreateCategoryInput, UpdateCategoryInput};
use tauri::State;

use crate::state::AppState;

// Phase 0 冻结的命令签名（对应 M2 分工清单 §3.2）。
// 命令名 + 入参/出参类型即前后端契约，变更需广播。
// M2-B1：各命令从 `State<AppState>` 取 category_repo，仅调用 trait 方法，不触碰具体存储实现。

/// 获取全部分类。
#[tauri::command]
pub fn list_categories(state: State<'_, AppState>) -> Result<Vec<Category>, AppError> {
    state.category_repo.find_all()
}

/// 创建分类。
#[tauri::command]
pub fn create_category(
    state: State<'_, AppState>,
    input: CreateCategoryInput,
) -> Result<Category, AppError> {
    state.category_repo.create(&input)
}

/// 更新分类（字段级局部更新）。
#[tauri::command]
pub fn update_category(
    state: State<'_, AppState>,
    id: String,
    input: UpdateCategoryInput,
) -> Result<Category, AppError> {
    state.category_repo.update(&id, &input)
}

/// 删除分类。
#[tauri::command]
pub fn delete_category(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.category_repo.delete(&id)
}
