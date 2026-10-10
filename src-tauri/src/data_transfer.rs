//! 数据导入 / 导出命令（对应《个人提示词管理平台导入导出设计方案》§6）。
//!
//! 面向「从一个用户导出、导入到另一个用户」的可移植交接：
//! - `export_data`：把当前库全量业务数据写为单个 JSON 文件，返回文件路径。
//! - `import_data`：读取 JSON 文件，校验后按「追加合并 / 覆盖清空」导入，返回报告。
//!
//! 导出文件为明文 JSON（不含账户/设备数据），无需保险库解锁；仅 CLI/Skills 的
//! 加密导出才走 `ensure_unlocked` 网关。

use std::fs;
use std::io::Read;
use std::path::Path;

use ipc::AppState;
use prompt_core::AppError;
use storage::transfer::{self, ImportReport};
use storage::Db;
use tauri::State;

fn io_err(e: std::io::Error) -> AppError {
    AppError::Io(e.to_string())
}

/// 导出全量业务数据到指定文件（JSON），返回写入的绝对/原始路径。
#[tauri::command]
pub fn export_data(db: State<'_, Db>, path: String) -> Result<String, AppError> {
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

/// 从平台 `.db` 数据库存储文件直接导入（对应「导入本平台 db 存储文件」需求）。
///
/// 明文库直接读；加密库用当前已解锁的 Master Key 打开（需保险库已解锁且口令一致）。
#[tauri::command]
pub fn import_db(
    db: State<'_, Db>,
    state: State<'_, AppState>,
    path: String,
    overwrite: bool,
) -> Result<ImportReport, AppError> {
    let src_path = Path::new(&path);
    let other = if is_encrypted_db(src_path)? {
        let key = state
            .vault
            .master_key()
            .map_err(|_| AppError::Auth("导入加密数据库前，请先解锁保险库".to_string()))?;
        Db::open_with_key(src_path, Some(&key))?
    } else {
        Db::open(src_path)?
    };
    let bundle = transfer::export_bundle(&other)?;
    transfer::validate(&bundle)?;
    transfer::import_bundle(&db, &bundle, overwrite)
}

/// 判断目标文件是否为 SQLCipher 加密库：明文 SQLite 以 `SQLite format 3\0` 魔数开头。
fn is_encrypted_db(path: &Path) -> Result<bool, AppError> {
    const MAGIC: &[u8; 16] = b"SQLite format 3\0";
    let mut f = fs::File::open(path).map_err(io_err)?;
    let mut head = [0u8; 16];
    let n = f.read(&mut head).map_err(io_err)?;
    if n < head.len() {
        // 文件过短不是合法库，视为明文交由打开阶段报错。
        return Ok(false);
    }
    Ok(&head != MAGIC)
}
