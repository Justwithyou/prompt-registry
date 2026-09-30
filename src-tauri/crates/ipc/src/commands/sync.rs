use crypto::derive_sync_key;
use prompt_core::{AppError, RemoteStore, SyncConfig, SyncResult, SyncStatus};
use sync::{WebDavClient, SYNC_DIR};
use tauri::State;

use crate::state::AppState;

// Phase 0 冻结的命令签名（对应 M4 分工清单 §3.2）。
// 命令名 + 入参/出参类型即前后端契约，变更需广播。
// M4-C 完整闭环：从 `State<AppState>` 取同步依赖，实现配置持久化 + 真正执行同步。

/// 配置 WebDAV 同步目标（持久化；若启用则验证远端可达并创建同步目录）。
#[tauri::command]
pub fn configure_sync(state: State<'_, AppState>, config: SyncConfig) -> Result<(), AppError> {
    state.ensure_unlocked()?;
    state.sync_config_store.save_config(&config)?;

    // 启用时尽早暴露网络 / 认证错误（MKCOL 已存在目录视为成功，幂等）。
    if config.enabled {
        let remote = WebDavClient::from_config(&config);
        remote.mkcol(SYNC_DIR)?;
    }
    Ok(())
}

/// 立即执行一次同步（拉取合并 + 推送）。
#[tauri::command]
pub fn sync_now(state: State<'_, AppState>) -> Result<SyncResult, AppError> {
    state.ensure_unlocked()?;

    let config = state
        .sync_config_store
        .load_config()?
        .ok_or_else(|| AppError::Sync("尚未配置同步目标".into()))?;
    if !config.enabled {
        return Err(AppError::Sync("同步未启用".into()));
    }

    // 从 Master Key 派生同步密钥（与数据库密钥域隔离），每次同步前注入。
    let master_key = state.vault.master_key()?;
    let sync_key = derive_sync_key(&master_key);
    state.sync_engine.set_sync_key(Some(sync_key))?;

    let remote = WebDavClient::from_config(&config);
    // 尽力创建远端目录（避开 WebDAV 服务未自动建目录的场景）。
    let _ = remote.mkcol(SYNC_DIR);
    state.sync_engine.sync_once(&remote)
}

/// 查询当前同步状态。
#[tauri::command]
pub fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatus, AppError> {
    state.ensure_unlocked()?;
    Ok(state.sync_engine.status())
}
