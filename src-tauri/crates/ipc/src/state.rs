use std::sync::Arc;

use prompt_core::{CategoryRepository, ProjectRepository, PromptRepository, TagRepository};

/// 应用全局状态（对应架构方案 4.3）。
///
/// M2 起扩展承载 project/tag/category 三个组织层 repository；
/// 由 `src-tauri/src/lib.rs` 在装配阶段构造具体实现并经 `app.manage(...)` 注入，
/// 各命令通过 `State<'_, AppState>` 取用业务能力。
/// crypto / sync 等依赖在 M4 引入后继续扩展。
pub struct AppState {
    pub prompt_repo: Arc<dyn PromptRepository>,
    pub project_repo: Arc<dyn ProjectRepository>,
    pub tag_repo: Arc<dyn TagRepository>,
    pub category_repo: Arc<dyn CategoryRepository>,
}
