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
        let mut conn = Connection::open(path).map_err(db_err)?;
        conn.migrate().map_err(db_err)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 获取底层连接锁（供 Repository 内部读写 / 开启事务）。
    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        self.conn
            .lock()
            .map_err(|_| AppError::Database("repository 连接锁已中毒".to_string()))
    }
}
