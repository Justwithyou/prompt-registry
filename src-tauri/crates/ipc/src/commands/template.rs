use prompt_core::{AppError, VariableDefinition};
use tauri::State;

use crate::state::AppState;

// Phase 0 冻结的命令签名（对应 M3 分工清单 §3.2）。
// 命令名 + 入参/出参类型即前后端契约，变更需广播。
// M3-B1：从 `State<AppState>` 取 template_extractor，调用 `TemplateExtractor::extract_variables`。

/// 从模板正文中提取 `{{变量名}}` 列表。
#[tauri::command]
pub fn extract_variables(
    state: State<'_, AppState>,
    content: String,
) -> Result<Vec<VariableDefinition>, AppError> {
    Ok(state.template_extractor.extract_variables(&content))
}
