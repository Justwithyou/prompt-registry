use std::path::Path;

use rusqlite::{Connection as SqliteConnection, Result as SqlResult};

/// 数据库连接封装（对应 M1-A1）。
///
/// 负责打开内存 / 文件 SQLite 连接，并可在首次使用时执行内置迁移。
pub struct Connection {
    inner: SqliteConnection,
}

impl Connection {
    /// 打开内存数据库（单元测试用）。
    pub fn open_in_memory() -> SqlResult<Self> {
        Ok(Self {
            inner: SqliteConnection::open_in_memory()?,
        })
    }

    /// 打开文件数据库。
    pub fn open(path: &Path) -> SqlResult<Self> {
        Ok(Self {
            inner: SqliteConnection::open(path)?,
        })
    }

    /// 暴露底层 rusqlite 连接（只读借用）。
    pub fn inner(&self) -> &SqliteConnection {
        &self.inner
    }

    /// 暴露底层 rusqlite 连接（可变借用，事务需要）。
    pub fn inner_mut(&mut self) -> &mut SqliteConnection {
        &mut self.inner
    }

    /// 执行内置迁移（幂等：已应用的版本跳过）。
    pub fn migrate(&mut self) -> SqlResult<()> {
        migrations::run(&mut self.inner)
    }
}

/// 版本化迁移：`(版本号, SQL)`，按版本号升序执行且只执行一次。
mod migrations {
    use rusqlite::{Connection, Result};

    const MIGRATIONS: &[(i64, &str)] = &[
        (1, include_str!("../../migrations/0001_init.sql")),
        (2, include_str!("../../migrations/0002_organize.sql")),
    ];

    pub fn run(conn: &mut Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version    INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;

        for &(version, sql) in MIGRATIONS {
            let applied: i64 = conn.query_row(
                "SELECT COUNT(*) FROM _migrations WHERE version = ?1",
                [version],
                |row| row.get(0),
            )?;
            if applied > 0 {
                continue;
            }

            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.execute("INSERT INTO _migrations (version) VALUES (?1)", [version])?;
            tx.commit()?;
        }

        Ok(())
    }
}
