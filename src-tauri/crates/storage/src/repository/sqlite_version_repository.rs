use std::path::Path;
use std::sync::MutexGuard;

use prompt_core::{AppError, Prompt, PromptVersion, VersionRepository};
use rusqlite::{OptionalExtension, Row};
use uuid::Uuid;

use crate::repository::sqlite_prompt_repository::{row_to_prompt, sync_fts, PROMPT_COLUMNS};
use crate::sqlite::Db;
use crate::util::{db_err, latest_version, next_version, now_timestamp};

/// 版本历史列清单，顺序固定，供 `row_to_version` 按序读取。
const VERSION_COLUMNS: &str = "id, prompt_id, version, content, change_summary, created_at";

/// SQLite 实现的版本历史仓库（M3-A4，明文 SQLite，rusqlite bundled）。
///
/// 与提示词仓库共享同一 [`Db`]，从而在同一事务 / 连接内完成「更新正文 + 追加版本」。
pub struct SqliteVersionRepository {
    db: Db,
}

impl SqliteVersionRepository {
    /// 打开内存数据库并完成迁移（单元测试用）。
    pub fn open_in_memory() -> Result<Self, AppError> {
        Ok(Self::new(Db::open_in_memory()?))
    }

    /// 打开文件数据库并完成迁移。
    pub fn open(path: &Path) -> Result<Self, AppError> {
        Ok(Self::new(Db::open(path)?))
    }

    /// 由共享连接构造仓库（供装配阶段复用同一 [`Db`]）。
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    fn lock(&self) -> Result<MutexGuard<'_, crate::sqlite::Connection>, AppError> {
        self.db.lock()
    }
}

impl VersionRepository for SqliteVersionRepository {
    fn list(&self, prompt_id: &str) -> Result<Vec<PromptVersion>, AppError> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {VERSION_COLUMNS} FROM prompt_versions WHERE prompt_id = ?1 \
             ORDER BY created_at DESC, rowid DESC"
        );
        let mut stmt = conn.inner().prepare(&sql).map_err(db_err)?;
        let rows = stmt
            .query_map([prompt_id], row_to_version)
            .map_err(db_err)?;
        rows.map(|r| r.map_err(db_err)).collect()
    }

    fn get(&self, version_id: &str) -> Result<PromptVersion, AppError> {
        let conn = self.lock()?;
        let sql = format!("SELECT {VERSION_COLUMNS} FROM prompt_versions WHERE id = ?1");
        conn.inner()
            .query_row(&sql, [version_id], row_to_version)
            .optional()
            .map_err(db_err)?
            .ok_or_else(|| AppError::NotFound(version_id.to_string()))
    }

    fn rollback(&self, prompt_id: &str, version_id: &str) -> Result<Prompt, AppError> {
        let mut conn = self.lock()?;

        // 读取目标历史版本，并校验其归属（防止跨提示词回滚）。
        let target_sql = format!("SELECT {VERSION_COLUMNS} FROM prompt_versions WHERE id = ?1");
        let target = conn
            .inner()
            .query_row(&target_sql, [version_id], row_to_version)
            .optional()
            .map_err(db_err)?
            .ok_or_else(|| AppError::NotFound(version_id.to_string()))?;
        if target.prompt_id != prompt_id {
            return Err(AppError::NotFound(version_id.to_string()));
        }

        // 读取当前提示词（已被软删除则视为不存在）。
        let prompt_sql =
            format!("SELECT {PROMPT_COLUMNS} FROM prompts WHERE id = ?1 AND deleted_at IS NULL");
        let mut prompt = conn
            .inner()
            .query_row(&prompt_sql, [prompt_id], row_to_prompt)
            .optional()
            .map_err(db_err)?
            .ok_or_else(|| AppError::NotFound(prompt_id.to_string()))?;

        // 追加式回滚：先归档回滚前的正文，再把正文替换为目标版本内容，生成一条新版本。
        let old_content = prompt.content.clone();
        prompt.content = target.content.clone();
        prompt.updated_at = now_timestamp();

        let version = next_version(latest_version(conn.inner(), prompt_id)?.as_deref());
        let version_id = Uuid::new_v4().to_string();
        let change_summary = format!("回滚到 {}", target.version);

        let tx = conn.inner_mut().transaction().map_err(db_err)?;
        tx.execute(
            "UPDATE prompts SET content = ?2, updated_at = ?3 WHERE id = ?1",
            rusqlite::params![prompt.id, prompt.content, prompt.updated_at],
        )
        .map_err(db_err)?;
        // 「生成新版本，不覆盖历史」：新版本记录回滚前的正文（与 update 的旧内容归档语义一致）。
        tx.execute(
            "INSERT INTO prompt_versions (id, prompt_id, version, content, change_summary, \
             created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                version_id,
                prompt.id,
                version,
                old_content,
                change_summary,
                prompt.updated_at,
            ],
        )
        .map_err(db_err)?;
        sync_fts(&tx, &prompt)?;
        tx.commit().map_err(db_err)?;

        Ok(prompt)
    }
}

fn row_to_version(row: &Row<'_>) -> rusqlite::Result<PromptVersion> {
    Ok(PromptVersion {
        id: row.get(0)?,
        prompt_id: row.get(1)?,
        version: row.get(2)?,
        content: row.get(3)?,
        change_summary: row.get(4)?,
        created_at: row.get(5)?,
    })
}
