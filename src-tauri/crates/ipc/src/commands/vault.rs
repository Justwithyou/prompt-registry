use prompt_core::AppError;
use tauri::State;

use crate::state::AppState;

// Phase 0 冻结的命令签名（对应 M4 分工清单 §3.2）。
// 命令名 + 入参/出参类型即前后端契约，变更需广播。
// M4-D1：从 `State<AppState>` 取 vault 依赖，转发到 `VaultProvider` trait 实现。

/// 首次设置主口令（数据库由明文升级为 SQLCipher 加密）。
///
/// 转发 `VaultProvider::setup`：派生 KEK、生成并包装 Master Key；
/// 具体 `PRAGMA rekey` 由轨道 B 在打开库时依据 Master Key 完成。
#[tauri::command]
pub fn setup_vault(state: State<'_, AppState>, password: String) -> Result<(), AppError> {
    state.vault.setup(&password)
}

/// 解锁保险库。
#[tauri::command]
pub fn unlock_vault(state: State<'_, AppState>, password: String) -> Result<(), AppError> {
    state.vault.unlock(&password)
}

/// 锁定保险库（清除内存 Master Key 与同步引擎密钥）。
#[tauri::command]
pub fn lock_vault(state: State<'_, AppState>) -> Result<(), AppError> {
    state.vault.lock()?;
    state.sync_engine.set_sync_key(None)
}

/// 修改主口令（仅重新包装 Master Key，数据密钥不变）。
#[tauri::command]
pub fn change_password(
    state: State<'_, AppState>,
    old_pw: String,
    new_pw: String,
) -> Result<(), AppError> {
    state.vault.change_password(&old_pw, &new_pw)
}

/// 查询是否处于锁定态。
///
/// 以「能否取到 Master Key」作为判定：未 setup（无信封）或已 lock 时取不到，
/// 均视为「锁定」；setup 且未 lock 时为「未锁定」。
#[tauri::command]
pub fn is_locked(state: State<'_, AppState>) -> Result<bool, AppError> {
    Ok(state.vault.master_key().is_err())
}
