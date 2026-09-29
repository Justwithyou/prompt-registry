//! crate 内共享工具函数（不对外导出）。

use prompt_core::AppError;
use rusqlite::OptionalExtension;

/// 将 rusqlite 错误折算为统一 `AppError`（对应架构方案 4.5 的错误折算约定）。
pub(crate) fn db_err(e: rusqlite::Error) -> AppError {
    AppError::Database(e.to_string())
}

/// 当前 UTC 时间戳（RFC3339，毫秒精度，供 `created_at` / `updated_at` 使用）。
pub(crate) fn now_timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// 查询某提示词最新的版本号（按插入顺序取最后一条）；无历史时返回 `None`。
pub(crate) fn latest_version(
    conn: &rusqlite::Connection,
    prompt_id: &str,
) -> Result<Option<String>, AppError> {
    conn.query_row(
        "SELECT version FROM prompt_versions WHERE prompt_id = ?1 ORDER BY rowid DESC LIMIT 1",
        [prompt_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(db_err)
}

/// 计算下一版本号（语义化 `主.次.补丁`）：无历史从 `0.1.0` 起，否则补丁 +1。
pub(crate) fn next_version(latest: Option<&str>) -> String {
    let Some(latest) = latest else {
        return "0.1.0".to_string();
    };

    let mut segments: Vec<u32> = latest
        .split('.')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect();
    // 保证至少三段（主.次.补丁）。
    while segments.len() < 3 {
        segments.push(0);
    }
    if let Some(patch) = segments.last_mut() {
        *patch += 1;
    }
    segments
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::next_version;

    #[test]
    fn first_version_is_default() {
        assert_eq!(next_version(None), "0.1.0");
    }

    #[test]
    fn increments_patch() {
        assert_eq!(next_version(Some("0.1.0")), "0.1.1");
        assert_eq!(next_version(Some("0.1.9")), "0.1.10");
        assert_eq!(next_version(Some("1.2.3")), "1.2.4");
    }
}
