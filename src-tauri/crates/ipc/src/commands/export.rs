use std::collections::HashMap;
use std::path::Path;

use export::{ExportFormat, ExportSelector, ExportSelectorInput, ExportService};
use prompt_core::AppError;
use tauri::State;

use crate::state::AppState;

// CLI/Skills 输出能力（对应《CLI 与 Skills 输出能力设计方案》§9）。
// 命令名 + 入参/出参类型即前后端契约；锁定态拒绝导出，需先解锁。

/// 从 AppState 复用已注入的 repository 与模板提取器，构造导出内核。
fn export_service(state: &AppState) -> ExportService {
    ExportService::new(
        state.prompt_repo.clone(),
        state.category_repo.clone(),
        state.tag_repo.clone(),
        state.project_repo.clone(),
        state.template_extractor.clone(),
    )
}

/// 渲染单条模板提示词为成品（变量值优先级：显式输入 > default_value）。
#[tauri::command]
pub fn render_prompt(
    state: State<'_, AppState>,
    id: String,
    vars: HashMap<String, String>,
) -> Result<String, AppError> {
    state.ensure_unlocked()?;
    export_service(&state).render_one(&id, &vars)
}

/// 按选择器（单条 / 分类 / 标签 / 项目）导出到目录，返回写入的文件路径列表。
#[tauri::command]
pub fn export_prompts(
    state: State<'_, AppState>,
    selector: ExportSelectorInput,
    format: String,
    out_dir: String,
    render: bool,
    vars: HashMap<String, String>,
) -> Result<Vec<String>, AppError> {
    state.ensure_unlocked()?;
    let fmt = ExportFormat::parse(&format).ok_or_else(|| AppError::Template(format!("未知导出格式: {format}")))?;
    export_service(&state).export(
        &selector.to_selector(),
        fmt,
        render,
        &vars,
        Path::new(&out_dir),
    )
}

/// 单条导出为 SKILL.md 目录，返回 SKILL.md 文件路径。
#[tauri::command]
pub fn export_skill(
    state: State<'_, AppState>,
    id: String,
    out_dir: String,
) -> Result<String, AppError> {
    state.ensure_unlocked()?;
    let paths = export_service(&state).export(
        &ExportSelector::Ids(vec![id]),
        ExportFormat::Skill,
        false,
        &HashMap::new(),
        Path::new(&out_dir),
    )?;
    Ok(paths.into_iter().next().unwrap_or_default())
}