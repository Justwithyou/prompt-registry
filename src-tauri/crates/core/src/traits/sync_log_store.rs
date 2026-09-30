use crate::errors::AppError;
use crate::models::SyncLogEntry;

/// 本地同步变更日志抽象（M4-C3）。
///
/// 生产由 storage 的 SQLite 实现接线（`SqliteSyncLogStore`），测试用内存实现。
/// 定义在 core 以避免「storage 实现本 trait → storage 依赖 sync」的依赖倒置。
pub trait SyncLogStore: Send + Sync {
    /// 记录一条本地变更。条目 id / HLC / device_id 由实现内部生成，
    /// 命令层无需感知同步时钟细节。
    fn record(
        &self,
        entity_type: &str,
        entity_id: &str,
        operation: &str,
        payload: &str,
    ) -> Result<(), AppError>;

    /// 采集全部待同步的本地变更日志（按写入顺序）。
    fn pending(&self) -> Result<Vec<SyncLogEntry>, AppError>;

    /// 把合并结果落地回本地数据（含跨设备 LWW 结果），按 `entity_type` 分发。
    fn apply(&self, entries: &[SyncLogEntry]) -> Result<(), AppError>;

    /// 成功推送后清除已同步的本地日志。
    fn mark_synced(&self, entries: &[SyncLogEntry]) -> Result<(), AppError>;
}
