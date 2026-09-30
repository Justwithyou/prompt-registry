use super::remote_store::RemoteStore;
use crate::errors::AppError;
use crate::models::{SyncLogEntry, SyncResult};

/// 同步引擎抽象（M4 分工清单 §3.2）。
///
/// 轨道 D 依赖它；A/B/C 各自实现内部细节（加密管道、WebDAV、CRDT 合并）。
pub trait SyncEngine: Send + Sync {
    /// 采集本地待同步的变更日志。
    fn collect_pending(&self) -> Result<Vec<SyncLogEntry>, AppError>;
    /// 与远端做一次双向同步（拉取合并 + 推送）。
    fn sync_once(&self, remote: &dyn RemoteStore) -> Result<SyncResult, AppError>;
}
