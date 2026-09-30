use std::sync::Arc;

use prompt_core::{
    AppError, CategoryRepository, ProjectRepository, PromptRepository, SyncEngine, TagRepository,
    TemplateExtractor, VaultProvider, VersionRepository,
};

/// 应用全局状态（对应架构方案 4.3）。
///
/// M2 起扩展承载 project/tag/category 三个组织层 repository；
/// M3 起再扩展 version_repo（版本历史）与 template_extractor（模板变量提取）；
/// M4 起注入 vault（密钥管理）与 sync_engine（同步引擎）。
/// 由 `src-tauri/src/lib.rs` 在装配阶段构造具体实现并经 `app.manage(...)` 注入，
/// 各命令通过 `State<'_, AppState>` 取用业务能力。
pub struct AppState {
    pub prompt_repo: Arc<dyn PromptRepository>,
    pub project_repo: Arc<dyn ProjectRepository>,
    pub tag_repo: Arc<dyn TagRepository>,
    pub category_repo: Arc<dyn CategoryRepository>,
    pub version_repo: Arc<dyn VersionRepository>,
    pub template_extractor: Arc<dyn TemplateExtractor>,
    /// 密钥保险库（`VaultProvider` 实现，来自 crypto crate）。
    pub vault: Arc<dyn VaultProvider>,
    /// 同步引擎（`SyncEngine` 实现，来自 sync crate）。
    ///
    /// 轨道 C 尚未实现 `SyncEngine`，暂以 `Option` 过渡注入 `None`；
    /// M4-C 完成后改为 `Arc<dyn SyncEngine>`（或直接注入具体实现）并移除 None 分支。
    pub sync_engine: Option<Arc<dyn SyncEngine>>,
}

impl AppState {
    /// 锁定态网关：数据 / 同步命令执行前校验保险库已解锁（M4-D2）。
    ///
    /// 锁定或未 setup 时 `vault.master_key()` 取不到 Master Key 而报错，
    /// 借此拦截在锁定态对加密库的读取，返回统一错误给前端。
    pub fn ensure_unlocked(&self) -> Result<(), AppError> {
        self.vault.master_key().map(|_| ())
    }
}