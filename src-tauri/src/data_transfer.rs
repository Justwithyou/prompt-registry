//! 数据导入 / 导出命令（对应《个人提示词管理平台导入导出设计方案》§6）。
//!
//! 面向「从一个用户导出、导入到另一个用户」的可移植交接：
//! - `export_data`：把当前库全量业务数据写为单个 JSON 文件，返回文件路径。
//! - `import_data`：读取 JSON 文件，校验后按「追加合并 / 覆盖清空」导入，返回报告。
//!
//! 导出文件为明文 JSON（不含账户/设备数据），无需保险库解锁；仅 CLI/Skills 的
//! 加密导出才走 `ensure_unlocked` 网关。

use std::fs;
use std::path::Path;

use prompt_core::AppError;
use storage::transfer::{self, ImportReport};
use storage::Db;
use tauri::State;

fn io_err(e: std::io::Error) -> AppError {
    AppError::Io(e.to_string())
}

/// 导出全量业务数据到指定文件（JSON），返回写入的绝对/原始路径。
#[tauri::command]
pub fn export_data(
    db: State<'_, Db>,
    path: String,
) -> Result<String, AppError> {
    let bundle = transfer::export_bundle(&db)?;
    let json = transfer::bundle_to_json(&bundle)?;
    let target = Path::new(&path);
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(io_err)?;
        }
    }
    fs::write(target, json).map_err(io_err)?;
    Ok(path)
}

/// 从 JSON 文件导入全量业务数据；`overwrite = true` 时先清空再导入，否则追加合并。
#[tauri::command]
pub fn import_data(
    db: State<'_, Db>,
    path: String,
    overwrite: bool,
) -> Result<ImportReport, AppError> {
    let text = fs::read_to_string(Path::new(&path)).map_err(io_err)?;
    let bundle = transfer::bundle_from_json(&text)?;
    transfer::validate(&bundle)?;
    transfer::import_bundle(&db, &bundle, overwrite)
}