//! crate 内共享工具函数（不对外导出）。

use prompt_core::AppError;

/// 将 rusqlite 错误折算为统一 `AppError`（对应架构方案 4.5 的错误折算约定）。
pub(crate) fn db_err(e: rusqlite::Error) -> AppError {
    AppError::Database(e.to_string())
}

/// 当前 UTC 时间戳（RFC3339，毫秒精度，供 `created_at` / `updated_at` 使用）。
pub(crate) fn now_timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
