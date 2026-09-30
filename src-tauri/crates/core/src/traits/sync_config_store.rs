use crate::errors::AppError;
use crate::models::SyncConfig;

/// WebDAV 同步配置持久化抽象（M4-C）。
///
/// `configure_sync` 写入、`sync_now` 读取，均面向本 trait；生产由 storage 的
/// SQLite 实现（单行 `sync_config` 表）接线。
pub trait SyncConfigStore: Send + Sync {
    /// 读取已保存的同步配置；未配置时返回 `None`。
    fn load_config(&self) -> Result<Option<SyncConfig>, AppError>;
    /// 保存（覆盖）同步配置。
    fn save_config(&self, config: &SyncConfig) -> Result<(), AppError>;
}
