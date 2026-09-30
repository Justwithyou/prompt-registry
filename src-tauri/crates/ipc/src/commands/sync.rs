use prompt_core::{AppError, SyncConfig, SyncResult, SyncStatus};
use tauri::State;

use crate::state::AppState;

// Phase 0 冻结的命令签名（对应 M4 分工清单 §3.2）。
// 命令名 + 入参/出参类型即前后端契约，变更需广播。
// M4-D1：从 `State<AppState>` 取 sync 依赖，实现同步流程。
//
// 说明：`SyncEngine` 由轨道 C（sync crate）实现，当前尚未就绪，
// AppState 以 `Option<Arc<dyn SyncEngine>>` 过渡注入 None。
// 命令先做锁定态校验，再在引擎缺失时返回明确的「未就绪」错误，
// 待 M4-C 完成并注入具体实现后，以下 TODO 接线点即启用。

/// 配置 WebDAV 同步目标。
#[tauri::command]
pub fn configure_sync(state: State<'_, AppState>, config: SyncConfig) -> Result<(), AppError> {
    let _ = config;
    state.ensure_unlocked()?;
    // TODO(M4-C 完成后接线)：将 `config` 持久化并交由 SyncEngine 使用。
    Err(AppError::Sync("同步引擎未就绪（轨道 C 尚未实现）".into()))
}

/// 立即执行一次同步（拉取合并 + 推送）。
#[tauri::command]
pub fn sync_now(state: State<'_, AppState>) -> Result<SyncResult, AppError> {
    state.ensure_unlocked()?;
    // TODO(M4-C 完成后接线)：构建 RemoteStore 并调用 `SyncEngine::sync_once`。
    Err(AppError::Sync("同步引擎未就绪（轨道 C 尚未实现）".into()))
}

/// 查询当前同步状态。
#[tauri::command]
pub fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatus, AppError> {
    state.ensure_unlocked()?;
    // TODO(M4-C 完成后接线)：返回引擎维护的同步状态。
    Err(AppError::Sync("同步引擎未就绪（轨道 C 尚未实现）".into()))
}