//! 同步引擎（M4-C3）：`SyncEngine` 实现，打通「本地变更 → 加密上传 → 下载解密 → LWW 合并回写」。
//!
//! 本地变更日志通过 [`SyncLogStore`] 抽象注入：生产由 IPC/存储层接线到 SQLite，
//! 测试用内存实现，从而不依赖 storage 具体实现（M4 分工清单 §6 边界）。

use std::sync::{Arc, RwLock};

use prompt_core::{
    AppError, EncryptedPayload, RemoteStore, SyncCipher, SyncEngine, SyncLogEntry, SyncLogStore,
    SyncResult, SyncStatus,
};

use crate::crdt::merge_entries;

/// 同步密钥长度（AES-256-GCM 的 256 位密钥）。
pub const SYNC_KEY_LEN: usize = 32;

/// 远端同步数据包所在目录。
pub const SYNC_DIR: &str = "prompt-manager";

/// 远端同步数据包路径（固定文件名，位于 [`SYNC_DIR`] 下）。
pub const SYNC_PATH: &str = "prompt-manager/sync-log.bin";

fn sync_err(msg: impl Into<String>) -> AppError {
    AppError::Sync(msg.into())
}

/// 同步引擎默认实现。
///
/// 持有加密管道（`SyncCipher`，来自轨道 A 的 trait）与本地日志仓库（`SyncLogStore`，
/// 来自 core 的 trait），两者均以 trait 对象注入，实现解耦。
pub struct Engine {
    cipher: Arc<dyn SyncCipher>,
    store: Arc<dyn SyncLogStore>,
    device_id: String,
    /// 同步密钥由 Master Key 派生，解锁后才可用，故以 `Option` 动态注入。
    sync_key: RwLock<Option<[u8; SYNC_KEY_LEN]>>,
    remote_path: String,
    last_sync_at: RwLock<Option<String>>,
}

impl Engine {
    /// 构造引擎；不持有口令，同步密钥经 [`Self::set_sync_key`] 在每次同步前注入。
    pub fn new(
        cipher: Arc<dyn SyncCipher>,
        store: Arc<dyn SyncLogStore>,
        device_id: impl Into<String>,
    ) -> Self {
        Self {
            cipher,
            store,
            device_id: device_id.into(),
            sync_key: RwLock::new(None),
            remote_path: SYNC_PATH.to_string(),
            last_sync_at: RwLock::new(None),
        }
    }

    /// 覆盖远端数据包路径（默认 [`SYNC_PATH`]）。
    pub fn with_remote_path(mut self, path: impl Into<String>) -> Self {
        self.remote_path = path.into();
        self
    }

    /// 当前设备 ID。
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// 取当前同步密钥；未注入（保险库尚未解锁）时返回错误。
    fn key(&self) -> Result<[u8; SYNC_KEY_LEN], AppError> {
        self.sync_key
            .read()
            .map_err(|_| sync_err("同步密钥状态锁已中毒"))?
            .ok_or_else(|| sync_err("同步密钥未就绪（保险库未解锁）"))
    }

    /// 加密本地日志并上传远端。
    fn upload(&self, remote: &dyn RemoteStore, entries: &[SyncLogEntry]) -> Result<(), AppError> {
        let key = self.key()?;
        let plaintext = serde_json::to_vec(entries).map_err(|e| sync_err(e.to_string()))?;
        let payload = self.cipher.encrypt(&plaintext, &key)?;
        let data = serde_json::to_vec(&payload).map_err(|e| sync_err(e.to_string()))?;
        remote.put(&self.remote_path, &data)
    }

    /// 下载远端数据包并解密；远端不存在时返回 `None`。
    fn download(&self, remote: &dyn RemoteStore) -> Result<Option<Vec<SyncLogEntry>>, AppError> {
        let data = match remote.get(&self.remote_path) {
            Ok(data) => data,
            Err(AppError::NotFound(_)) => return Ok(None),
            Err(e) => return Err(e),
        };
        let key = self.key()?;
        let payload: EncryptedPayload =
            serde_json::from_slice(&data).map_err(|e| sync_err(e.to_string()))?;
        let plaintext = self.cipher.decrypt(&payload, &key)?;
        let entries: Vec<SyncLogEntry> =
            serde_json::from_slice(&plaintext).map_err(|e| sync_err(e.to_string()))?;
        Ok(Some(entries))
    }
}

impl SyncEngine for Engine {
    fn set_sync_key(&self, key: Option<[u8; SYNC_KEY_LEN]>) -> Result<(), AppError> {
        let mut guard = self
            .sync_key
            .write()
            .map_err(|_| sync_err("同步密钥状态锁已中毒"))?;
        *guard = key;
        Ok(())
    }

    fn collect_pending(&self) -> Result<Vec<SyncLogEntry>, AppError> {
        self.store.pending()
    }

    fn status(&self) -> SyncStatus {
        let last_sync_at = self.last_sync_at.read().ok().and_then(|g| g.clone());
        SyncStatus {
            last_sync_at,
            pending_changes: self.store.pending().map(|v| v.len() as u32).unwrap_or(0),
            device_id: self.device_id.clone(),
        }
    }

    fn sync_once(&self, remote: &dyn RemoteStore) -> Result<SyncResult, AppError> {
        let pending = self.collect_pending()?;
        let pushed = pending.len() as u32;

        // 1. 先拉取远端并解密（避免「先传后回读自己」造成合并计数虚高）。
        let remote_entries = self.download(remote)?.unwrap_or_default();
        let pulled = remote_entries.len() as u32;

        // 2. 冲突合并（字段级 LWW）得到合意状态，并写回本地。
        let merged_entries = if remote_entries.is_empty() {
            Vec::new()
        } else {
            merge_entries(&pending, &remote_entries)
        };
        let merged = merged_entries.len() as u32;
        if !merged_entries.is_empty() {
            self.store.apply(&merged_entries)?;
        }

        // 3. 上传合意状态（远端为空时即本地 pending），保持远端与本地一致。
        let to_upload: &[SyncLogEntry] = if merged_entries.is_empty() {
            pending.as_slice()
        } else {
            merged_entries.as_slice()
        };
        self.upload(remote, to_upload)?;

        // 4. 清除已成功推送的本地日志。
        if !pending.is_empty() {
            self.store.mark_synced(&pending)?;
        }

        if let Ok(mut guard) = self.last_sync_at.write() {
            *guard = Some(chrono::Utc::now().to_rfc3339());
        }

        Ok(SyncResult {
            pushed,
            pulled,
            merged,
        })
    }
}
