use std::path::{Path, PathBuf};

use rusqlite::{Connection as SqliteConnection, Result as SqlResult};

/// 数据库连接封装（对应 M1-A1）。
///
/// 负责打开内存 / 文件 SQLite 连接，并可在首次使用时执行内置迁移。
pub struct Connection {
    inner: SqliteConnection,
    /// 打开时对应的文件路径（内存库为 `None`），供 M4 明文→加密迁移使用。
    path: Option<PathBuf>,
}

impl Connection {
    /// 打开内存数据库（单元测试用）。
    pub fn open_in_memory() -> SqlResult<Self> {
        Ok(Self {
            inner: SqliteConnection::open_in_memory()?,
            path: None,
        })
    }

    /// 打开文件数据库。
    pub fn open(path: &Path) -> SqlResult<Self> {
        Ok(Self {
            inner: SqliteConnection::open(path)?,
            path: Some(path.to_path_buf()),
        })
    }

    /// 以 SQLCipher 原始密钥打开加密数据库（M4-B2）。
    ///
    /// 打开后立即执行 `PRAGMA key`，随后方可读取加密内容。
    pub fn open_with_key(path: &Path, key: &[u8; 32]) -> SqlResult<Self> {
        let mut conn = Self::open(path)?;
        conn.set_key(key)?;
        Ok(conn)
    }

    /// 设置当前连接的加密密钥（`PRAGMA key`），须在首次访问数据库前调用。
    pub fn set_key(&mut self, key: &[u8; 32]) -> SqlResult<()> {
        self.inner
            .execute_batch(&format!("PRAGMA key = \"{}\";", key_literal(key)))
    }

    /// 明文库就地升级为加密库，旧数据不丢（M4-B2）。
    ///
    /// SQLCipher 4 不允许对明文库直接 `PRAGMA rekey`；需用 `sqlcipher_export()`
    /// 把当前明文库导出到一个带密钥的临时加密库，再以该文件替换原文件并重开。
    pub fn rekey(&mut self, key: &[u8; 32]) -> SqlResult<()> {
        let path = self.path.clone().ok_or(rusqlite::Error::InvalidQuery)?;

        // 同目录临时加密库，避免跨卷 rename 失败。
        let mut os = path.as_os_str().to_owned();
        os.push(".rekey-tmp");
        let temp = PathBuf::from(os);
        let _ = std::fs::remove_file(&temp);

        // 1) 挂载带目标密钥的临时库，并把明文 main 的内容导出成加密库。
        let temp_sql = temp.display().to_string().replace('\'', "''");
        self.inner.execute_batch(&format!(
            "ATTACH DATABASE '{}' AS encrypted KEY \"{}\";",
            temp_sql,
            key_literal(key)
        ))?;
        self.inner
            .execute_batch("SELECT sqlcipher_export('encrypted');")?;
        self.inner.execute_batch("DETACH DATABASE encrypted;")?;

        // 2) 关闭原始明文连接，再用加密临时文件替换原文件（Windows 需释放句柄）。
        drop(std::mem::replace(
            &mut self.inner,
            SqliteConnection::open_in_memory()?,
        ));
        std::fs::remove_file(&path)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        std::fs::rename(&temp, &path)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        // 3) 以密钥重开，保证后续同一连接即为加密连接。
        self.inner = SqliteConnection::open(&path)?;
        self.set_key(key)
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

/// 将 32 字节 Master Key 序列化为 SQLCipher 原始密钥字面量 `x'<64 位 hex>'`。
fn key_literal(key: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(2 + key.len() * 2);
    s.push_str("x'");
    for b in key {
        let _ = write!(s, "{b:02x}");
    }
    s.push('\'');
    s
}

/// 版本化迁移：`(版本号, SQL)`，按版本号升序执行且只执行一次。
mod migrations {
    use rusqlite::{Connection, Result};

    const MIGRATIONS: &[(i64, &str)] = &[
        (1, include_str!("../../migrations/0001_init.sql")),
        (2, include_str!("../../migrations/0002_organize.sql")),
        (3, include_str!("../../migrations/0003_versioning.sql")),
        (4, include_str!("../../migrations/0004_sync_log.sql")),
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
