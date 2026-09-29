use std::sync::Arc;

use prompt_core::PromptRepository;

/// 应用全局状态（对应架构方案 4.3）。
///
/// M1 起由 `src-tauri/src/lib.rs` 在装配阶段构造 `SqlitePromptRepository` 并经
/// `app.manage(...)` 注入，各命令通过 `State<'_, AppState>` 取用业务能力。
///
/// M1 仅承载 `prompt_repo`；crypto / sync 等依赖在 M4 引入后扩展现有字段。
pub struct AppState {
    pub prompt_repo: Arc<dyn PromptRepository>,
}
