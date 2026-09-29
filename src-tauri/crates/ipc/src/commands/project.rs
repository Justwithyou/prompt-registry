use prompt_core::{AppError, CreateProjectInput, Project, UpdateProjectInput};
use tauri::State;

use crate::state::AppState;

// Phase 0 冻结的命令签名（对应 M2 分工清单 §3.2）。
// 命令名 + 入参/出参类型即前后端契约，变更需广播。
// M2-B1：各命令从 `State<AppState>` 取 project_repo，仅调用 trait 方法，不触碰具体存储实现。

/// 获取全部项目。
#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, AppError> {
    state.project_repo.find_all()
}

/// 创建项目。
#[tauri::command]
pub fn create_project(
    state: State<'_, AppState>,
    input: CreateProjectInput,
) -> Result<Project, AppError> {
    state.project_repo.create(&input)
}

/// 更新项目（字段级局部更新）。
#[tauri::command]
pub fn update_project(
    state: State<'_, AppState>,
    id: String,
    input: UpdateProjectInput,
) -> Result<Project, AppError> {
    state.project_repo.update(&id, &input)
}

/// 删除项目。
#[tauri::command]
pub fn delete_project(state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.project_repo.delete(&id)
}
