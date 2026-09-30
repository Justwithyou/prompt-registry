use serde::{Deserialize, Serialize};

/// 同步日志条目（本地变更的持久化记录，M4 分工清单 §3.2）。
///
/// 序列化契约：`rename_all = "camelCase"`。仅在后端同步/合并流程内部流转，不直接跨 IPC。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncLogEntry {
    pub id: String,
    /// 实体类型（prompt / category / tag / project / version 等）。
    pub entity_type: String,
    pub entity_id: String,
    /// 操作类型：create / update / delete。
    pub operation: String,
    /// 变更后的字段数据（JSON 序列化）。
    pub payload: String,
    /// HLC 时间戳编码（`wall_time-logical`），用于字段级 LWW 合并。
    pub hlc: String,
    pub device_id: String,
}

/// WebDAV 同步配置（`configure_sync` 入参）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SyncConfig {
    pub url: String,
    pub username: String,
    /// 可选令牌；缺省时回退 Basic Auth（用户名 + 密码）。
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub enabled: bool,
}

/// 同步状态（`get_sync_status` 返回）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub last_sync_at: Option<String>,
    pub pending_changes: u32,
    pub device_id: String,
}

/// 单次同步结果（`sync_now` 返回）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub pushed: u32,
    pub pulled: u32,
    pub merged: u32,
}
