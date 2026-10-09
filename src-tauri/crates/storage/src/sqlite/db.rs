use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use prompt_core::AppError;

use super::connection::Connection;
use crate::util::db_err;

/// 可共享的数据库句柄（M2 组织层）。
///
/// 多个 Repository（提示词 / 项目 / 标签 / 分类）共享同一个 SQLite 连接，
/// 避免多连接下的锁竞争；对 `:memory:` 测试而言，共享一个连接也保证各仓库
/// 看到同一份数据（否则各自 `open_in_memory()` 会得到相互隔离的库）。
#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl Db {
    /// 打开内存数据库并完成迁移（单元测试用）。
    pub fn open_in_memory() -> Result<Self, AppError> {
        let mut conn = Connection::open_in_memory().map_err(db_err)?;
        conn.migrate().map_err(db_err)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 打开文件数据库并完成迁移。
    pub fn open(path: &Path) -> Result<Self, AppError> {
        Self::open_with_key(path, None)
    }

    /// 以可选密钥打开文件数据库并完成迁移（M4-B2）。
    ///
    /// `Some(key)` → SQLCipher 加密库（`PRAGMA key`）；`None` → 明文打开。
    /// 是否已设口令由上层（装配阶段）依据保险库状态决定。
    pub fn open_with_key(path: &Path, key: Option<&[u8; 32]>) -> Result<Self, AppError> {
        let mut conn = match key {
            Some(k) => Connection::open_with_key(path, k).map_err(db_err)?,
            None => Connection::open(path).map_err(db_err)?,
        };
        conn.migrate().map_err(db_err)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 将已打开的明文库就地迁移为加密库（`PRAGMA rekey`，M4-B2）。
    ///
    /// 供首次设置口令时调用；调用后旧数据仍在，但需以同一密钥重新打开方可读取。
    pub fn rekey(&self, key: &[u8; 32]) -> Result<(), AppError> {
        let mut conn = self.lock()?;
        conn.rekey(key).map_err(db_err)
    }

    /// 获取底层连接锁（供 Repository 内部读写 / 开启事务）。
    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        self.conn
            .lock()
            .map_err(|_| AppError::Database("repository 连接锁已中毒".to_string()))
    }

    /// 将当前库的一致性快照导出到目标文件（`VACUUM INTO`），用于迁移存储位置。
    ///
    /// 目标文件不得已存在（SQLite 约束），由调用方在迁移前校验。
    pub fn vacuum_into(&self, target: &Path) -> Result<(), AppError> {
        let conn = self.lock()?;
        let sql = format!(
            "VACUUM INTO '{}'",
            target.display().to_string().replace('\'', "''")
        );
        conn.inner().execute_batch(&sql).map_err(db_err)
    }

    /// 关闭当前连接并在 `path` 重开（含幂等迁移），用于运行时切换存储位置。
    ///
    /// 共享连接的多个 Repository 因持有同一个 `Arc<Mutex<Connection>>`，
    /// 重开后即全体指向新库，无需重建仓库实例。
    pub fn reopen(&self, path: &Path) -> Result<(), AppError> {
        let mut conn = self.lock()?;
        let mut next = Connection::open(path).map_err(db_err)?;
        next.migrate().map_err(db_err)?;
        *conn = next;
        Ok(())
    }
}
