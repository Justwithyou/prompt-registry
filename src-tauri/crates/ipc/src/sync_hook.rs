//! 同步采集钩子：命令层在成功变更实体后，把变更写入本地同步日志。
//!
//! 采集为尽力而为——同步日志写入失败不阻断主 CRUD 操作（同步是辅助能力），
//! 条目 id / HLC / device_id 由 `SyncLogStore` 实现内部生成。

use serde::Serialize;

use crate::state::AppState;

/// 记录一条「完整实体 JSON」的变更（create / update）。
pub fn record_json<T: Serialize>(
    state: &AppState,
    entity_type: &str,
    entity_id: &str,
    operation: &str,
    entity: &T,
) {
    if let Ok(payload) = serde_json::to_string(entity) {
        let _ = state
            .sync_log_store
            .record(entity_type, entity_id, operation, &payload);
    }
}

/// 记录一条删除变更（墓碑）。
pub fn record_delete(state: &AppState, entity_type: &str, entity_id: &str) {
    let _ = state
        .sync_log_store
        .record(entity_type, entity_id, "delete", "{}");
}
