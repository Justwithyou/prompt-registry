//! 存储位置管理：配置持久化 + 数据迁移 + 运行时热切换。
//!
//! 数据目录默认位于应用数据目录（`%APPDATA%\com.promptmanager.app`），
//! 用户可在设置中把它改到任意目录。配置保存在应用数据目录下的 `storage.json`（固定不动），
//! 数据库文件始终名为 `prompt.db`，仅所在目录可变。

use std::fs;
use std::path::{Path, PathBuf};

use prompt_core::AppError;
use serde::{Deserialize, Serialize};
use storage::Db;
use tauri::{AppHandle, Manager, State};

/// 存储位置配置文件名（位于应用数据目录，不随数据库一起移动）。
const CONFIG_FILE: &str = "storage.json";

#[derive(Debug, Default, Serialize, Deserialize)]
struct StorageConfig {
    /// 数据库所在目录；缺省时回退到应用数据目录。
    #[serde(rename = "dataDir", default)]
    data_dir: Option<String>,
}

/// 读取当前生效的数据存储目录；未配置 / 配置非法 / 目录不存在时回退到 `base`。
pub fn load_dir(base: &Path) -> PathBuf {
    if let Ok(text) = fs::read_to_string(base.join(CONFIG_FILE)) {
        if let Ok(cfg) = serde_json::from_str::<StorageConfig>(&text) {
            if let Some(dir) = cfg.data_dir {
                let p = normalize_path(Path::new(&dir));
                if p.is_dir() {
                    return p;
                }
            }
        }
    }
    base.to_path_buf()
}

fn save_dir(base: &Path, dir: &Path) -> Result<(), AppError> {
    let cfg = StorageConfig {
        data_dir: Some(dir.to_string_lossy().into_owned()),
    };
    let text = serde_json::to_string_pretty(&cfg).map_err(|e| AppError::Io(e.to_string()))?;
    fs::write(base.join(CONFIG_FILE), text).map_err(io_err)
}

fn io_err(e: std::io::Error) -> AppError {
    AppError::Io(e.to_string())
}

fn path_err(e: tauri::Error) -> AppError {
    AppError::Io(e.to_string())
}

/// 去除 Windows 长路径/设备前缀（`\\?\`、`\\.\`），用于显示与持久化。
///
/// `Path::canonicalize` 在 Windows 上会返回带 `\\?\` 前缀的「逐字路径」，
/// 直接展示/回读会出现 `\\?\D:\...` 这样的多余前缀，故统一归一化。
fn normalize_path(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    let s = s
        .strip_prefix("\\\\?\\")
        .or_else(|| s.strip_prefix("\\\\.\\"))
        .unwrap_or(&s);
    PathBuf::from(s)
}

/// 查询当前数据存储位置（目录路径）。
#[tauri::command]
pub fn get_storage_location(app: AppHandle) -> Result<String, AppError> {
    let base = app.path().app_data_dir().map_err(path_err)?;
    Ok(normalize_path(&load_dir(&base))
        .to_string_lossy()
        .into_owned())
}

/// 修改数据存储位置：校验并创建目录 → 复制当前库到新位置（保留旧库）→ 持久化配置 → 热切换。
///
/// 返回新目录的绝对路径。
#[tauri::command]
pub fn set_storage_location(
    app: AppHandle,
    db: State<'_, Db>,
    dir: String,
) -> Result<String, AppError> {
    let base = app.path().app_data_dir().map_err(path_err)?;
    let current = load_dir(&base);

    let trimmed = dir.trim();
    if trimmed.is_empty() {
        return Err(AppError::Io("存储路径不能为空".into()));
    }

    let new_dir = PathBuf::from(trimmed);
    fs::create_dir_all(&new_dir).map_err(io_err)?;
    let new_dir = new_dir.canonicalize().map_err(io_err)?;
    let display_dir = normalize_path(&new_dir);

    if current
        .canonicalize()
        .map(|c| c == new_dir)
        .unwrap_or(false)
    {
        return Ok(display_dir.to_string_lossy().into_owned());
    }

    let new_db = new_dir.join("prompt.db");
    if new_db.exists() {
        return Err(AppError::Io(
            "目标目录已存在 prompt.db，请选择不含提示词数据库的目录".into(),
        ));
    }

    // 1) 把当前库的一致性快照复制到新位置（VACUUM INTO，旧库保留不动）。
    db.vacuum_into(&new_db)?;
    // 2) 持久化新存储目录（供下次启动读取）。
    save_dir(&base, &display_dir)?;
    // 3) 运行时热切换到新库，所有 Repository 立即指向新文件。
    db.reopen(&new_db)?;

    Ok(display_dir.to_string_lossy().into_owned())
}
