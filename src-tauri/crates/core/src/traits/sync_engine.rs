use super::remote_store::RemoteStore;
use crate::errors::AppError;
use crate::models::{SyncLogEntry, SyncResult, SyncStatus};

/// 同步引擎抽象（M4 分工清单 §3.2）。
///
/// 轨道 D 依赖它；A/B/C 各自实现内部细节（加密管道、WebDAV、CRDT 合并）。
pub trait SyncEngine: Send + Sync {
    /// 设置 / 清除本次同步使用的密钥（`None` 表示保险库已锁定）。
    ///
    /// 密钥由装配层从 Master Key 派生（HKDF），因 Master Key 仅在 setup/unlock
    /// 后可用，故在 `sync_once` 前动态注入，而非引擎构造时固化。
    fn set_sync_key(&self, key: Option<[u8; 32]>) -> Result<(), AppError>;

    /// 采集本地待同步的变更日志。
    fn collect_pending(&self) -> Result<Vec<SyncLogEntry>, AppError>;

    /// 与远端做一次双向同步（拉取合并 + 推送）。
    fn sync_once(&self, remote: &dyn RemoteStore) -> Result<SyncResult, AppError>;

    /// 当前同步状态快照（`get_sync_status` 数据源）。
    fn status(&self) -> SyncStatus;
}
