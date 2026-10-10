use std::path::Path;

use prompt_core::{AppError, VaultEnvelope};
use storage::Db;
use tauri::State;

use crate::state::AppState;

// Phase 0 冻结的命令签名（对应 M4 分工清单 §3.2）。
// 命令名 + 入参/出参类型即前后端契约，变更需广播。
// M4-D1：从 `State<AppState>` 取 vault 依赖，转发到 `VaultProvider` trait 实现。
// M4 装配收尾（遗留 #2/#3）：setup 联动 `Db::rekey` 把明文库就地加密并持久化信封；
// unlock 联动 `Db::reopen_with_key` 把锁定占位库重开为真实加密库。

/// 首次设置主口令：生成并包装 Master Key，把明文库就地升级为 SQLCipher 加密库，并持久化解锁信封。
#[tauri::command]
pub fn setup_vault(
    state: State<'_, AppState>,
    db: State<'_, Db>,
    password: String,
) -> Result<(), AppError> {
    state.vault.setup(&password)?;
    let key = state.vault.master_key()?;
    // 明文库就地加密（SQLCipher rekey），旧数据不丢；调用后当前连接即为加密连接。
    db.rekey(&key)?;
    // 持久化解锁信封（盐 / nonce / 密文），重启后可据此解锁。
    if let Some(envelope) = state.vault.export_envelope() {
        write_envelope(&state.vault_file, &envelope)?;
    }
    Ok(())
}

/// 解锁保险库，并把启动时的锁定占位库重开为真实加密库。
#[tauri::command]
pub fn unlock_vault(
    state: State<'_, AppState>,
    db: State<'_, Db>,
    password: String,
) -> Result<(), AppError> {
    state.vault.unlock(&password)?;
    let key = state.vault.master_key()?;
    db.reopen_with_key(&state.db_path, &key)
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
    state.vault.change_password(&old_pw, &new_pw)?;
    // 重新包装后信封变化，需回写持久化。
    if let Some(envelope) = state.vault.export_envelope() {
        write_envelope(&state.vault_file, &envelope)?;
    }
    Ok(())
}

/// 查询是否处于锁定态。
///
/// 以「能否取到 Master Key」作为判定：未 setup（无信封）或已 lock 时取不到，
/// 均视为「锁定」；setup 且未 lock 时为「未锁定」。
#[tauri::command]
pub fn is_locked(state: State<'_, AppState>) -> Result<bool, AppError> {
    Ok(state.vault.master_key().is_err())
}

/// 控制 OS 钥匙串自动解锁：`true` 写入 Master Key（需已解锁），`false` 清除。
#[tauri::command]
pub fn set_auto_unlock(state: State<'_, AppState>, enabled: bool) -> Result<(), AppError> {
    state.vault.set_auto_unlock(enabled)
}

fn write_envelope(path: &Path, envelope: &VaultEnvelope) -> Result<(), AppError> {
    let json = serde_json::to_string(envelope).map_err(|e| AppError::Crypto(e.to_string()))?;
    std::fs::write(path, json).map_err(|e| AppError::Io(e.to_string()))
}
