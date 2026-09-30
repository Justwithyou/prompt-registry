//! 同步变更日志的 SQLite 实现（M4-C 完整闭环）。
//!
//! 同时实现两个 core trait：
//! - [`SyncLogStore`]：record / pending / apply / mark_synced，打通
//!   「本地变更 → 加密上传 → 合并回写 → 清除已同步」。
//! - [`SyncConfigStore`]：WebDAV 配置的单行持久化。
//!
//! `apply` 按 `entity_type` 分发到五类实体的 upsert / 删除（提示词同步 FTS 索引）。

use std::sync::MutexGuard;

use prompt_core::{
    AppError, Category, Project, Prompt, PromptVersion, SyncConfig, SyncConfigStore, SyncLogEntry,
    SyncLogStore, Tag,
};
use rusqlite::{params, OptionalExtension, Row};
use uuid::Uuid;

use crate::repository::sqlite_prompt_repository::{delete_fts, sync_fts};
use crate::sqlite::Db;
use crate::util::{db_err, now_timestamp};

const DEVICE_ID_KEY: &str = "device_id";

/// 生成当前设备的 HLC 时间戳（`<wall_millis>-0`；同一设备毫秒内编辑靠稳定排序兜底）。
fn current_hlc() -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    format!("{millis}-0")
}

fn sync_err(msg: impl std::fmt::Display) -> AppError {
    AppError::Sync(msg.to_string())
}

/// SQLite 实现的同步变更日志仓库。
pub struct SqliteSyncLogStore {
    db: Db,
    device_id: String,
}

impl SqliteSyncLogStore {
    /// 由共享连接构造；自动读取或生成稳定的 `device_id`。
    pub fn new(db: Db) -> Result<Self, AppError> {
        let device_id = Self::load_or_create_device_id(&db)?;
        Ok(Self { db, device_id })
    }

    /// 当前设备标识（跨重启稳定）。
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    fn lock(&self) -> Result<MutexGuard<'_, crate::sqlite::Connection>, AppError> {
        self.db.lock()
    }

    fn load_or_create_device_id(db: &Db) -> Result<String, AppError> {
        let conn = db.lock()?;
        let existing: Option<String> = conn
            .inner()
            .query_row(
                "SELECT value FROM sync_meta WHERE key = ?1",
                [DEVICE_ID_KEY],
                |row| row.get(0),
            )
            .optional()
            .map_err(db_err)?;
        if let Some(id) = existing {
            return Ok(id);
        }
        let id = Uuid::new_v4().to_string();
        conn.inner()
            .execute(
                "INSERT INTO sync_meta (key, value) VALUES (?1, ?2)",
                params![DEVICE_ID_KEY, id],
            )
            .map_err(db_err)?;
        Ok(id)
    }

    fn apply_one(&self, entry: &SyncLogEntry) -> Result<(), AppError> {
        match entry.entity_type.as_str() {
            "prompt" => self.apply_prompt(entry),
            "category" => self.apply_category(entry),
            "project" => self.apply_project(entry),
            "tag" => self.apply_tag(entry),
            "version" => self.apply_version(entry),
            other => Err(sync_err(format!("未知实体类型: {other}"))),
        }
    }

    fn apply_prompt(&self, entry: &SyncLogEntry) -> Result<(), AppError> {
        let mut conn = self.lock()?;
        let tx = conn.inner_mut().transaction().map_err(db_err)?;

        if entry.operation == "delete" {
            let now = now_timestamp();
            tx.execute(
                "UPDATE prompts SET deleted_at = ?2, updated_at = ?2 WHERE id = ?1",
                params![entry.entity_id, now],
            )
            .map_err(db_err)?;
            delete_fts(&tx, &entry.entity_id)?;
            tx.commit().map_err(db_err)?;
            return Ok(());
        }

        let prompt: Prompt = serde_json::from_str(&entry.payload)
            .map_err(|e| sync_err(format!("prompt payload 反序列化失败: {e}")))?;
        tx.execute(
            "INSERT INTO prompts (id, title, content, description, category, category_id, \
             is_pinned, is_favorite, rating, created_at, updated_at, deleted_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12) \
             ON CONFLICT(id) DO UPDATE SET \
               title = excluded.title, \
               content = excluded.content, \
               description = excluded.description, \
               category = excluded.category, \
               category_id = excluded.category_id, \
               is_pinned = excluded.is_pinned, \
               is_favorite = excluded.is_favorite, \
               rating = excluded.rating, \
               created_at = excluded.created_at, \
               updated_at = excluded.updated_at, \
               deleted_at = excluded.deleted_at",
            params![
                prompt.id,
                prompt.title,
                prompt.content,
                prompt.description,
                prompt.category,
                prompt.category_id,
                prompt.is_pinned,
                prompt.is_favorite,
                prompt.rating,
                prompt.created_at,
                prompt.updated_at,
                prompt.deleted_at,
            ],
        )
        .map_err(db_err)?;
        sync_fts(&tx, &prompt)?;
        tx.commit().map_err(db_err)?;
        Ok(())
    }

    fn apply_category(&self, entry: &SyncLogEntry) -> Result<(), AppError> {
        let mut conn = self.lock()?;
        let tx = conn.inner_mut().transaction().map_err(db_err)?;

        if entry.operation == "delete" {
            tx.execute(
                "DELETE FROM categories WHERE id = ?1",
                params![entry.entity_id],
            )
            .map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            return Ok(());
        }

        let cat: Category = serde_json::from_str(&entry.payload)
            .map_err(|e| sync_err(format!("category payload 反序列化失败: {e}")))?;
        tx.execute(
            "INSERT INTO categories (id, name, parent_id, sort_order, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5) \
             ON CONFLICT(id) DO UPDATE SET \
               name = excluded.name, \
               parent_id = excluded.parent_id, \
               sort_order = excluded.sort_order, \
               created_at = excluded.created_at",
            params![
                cat.id,
                cat.name,
                cat.parent_id,
                cat.sort_order,
                cat.created_at
            ],
        )
        .map_err(db_err)?;
        tx.commit().map_err(db_err)?;
        Ok(())
    }

    fn apply_project(&self, entry: &SyncLogEntry) -> Result<(), AppError> {
        let mut conn = self.lock()?;
        let tx = conn.inner_mut().transaction().map_err(db_err)?;

        if entry.operation == "delete" {
            tx.execute(
                "DELETE FROM projects WHERE id = ?1",
                params![entry.entity_id],
            )
            .map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            return Ok(());
        }

        let project: Project = serde_json::from_str(&entry.payload)
            .map_err(|e| sync_err(format!("project payload 反序列化失败: {e}")))?;
        tx.execute(
            "INSERT INTO projects (id, name, description, color, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT(id) DO UPDATE SET \
               name = excluded.name, \
               description = excluded.description, \
               color = excluded.color, \
               created_at = excluded.created_at, \
               updated_at = excluded.updated_at",
            params![
                project.id,
                project.name,
                project.description,
                project.color,
                project.created_at,
                project.updated_at,
            ],
        )
        .map_err(db_err)?;
        tx.commit().map_err(db_err)?;
        Ok(())
    }

    fn apply_tag(&self, entry: &SyncLogEntry) -> Result<(), AppError> {
        let mut conn = self.lock()?;
        let tx = conn.inner_mut().transaction().map_err(db_err)?;

        if entry.operation == "delete" {
            tx.execute("DELETE FROM tags WHERE id = ?1", params![entry.entity_id])
                .map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            return Ok(());
        }

        let tag: Tag = serde_json::from_str(&entry.payload)
            .map_err(|e| sync_err(format!("tag payload 反序列化失败: {e}")))?;
        tx.execute(
            "INSERT INTO tags (id, name, parent_id, color, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5) \
             ON CONFLICT(id) DO UPDATE SET \
               name = excluded.name, \
               parent_id = excluded.parent_id, \
               color = excluded.color, \
               created_at = excluded.created_at",
            params![tag.id, tag.name, tag.parent_id, tag.color, tag.created_at],
        )
        .map_err(db_err)?;
        tx.commit().map_err(db_err)?;
        Ok(())
    }

    fn apply_version(&self, entry: &SyncLogEntry) -> Result<(), AppError> {
        // 版本历史不可变（追加式），无删除语义。
        if entry.operation == "delete" {
            return Ok(());
        }
        let version: PromptVersion = serde_json::from_str(&entry.payload)
            .map_err(|e| sync_err(format!("version payload 反序列化失败: {e}")))?;
        let conn = self.lock()?;
        conn.inner()
            .execute(
                "INSERT OR IGNORE INTO prompt_versions (id, prompt_id, version, content, \
                 change_summary, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    version.id,
                    version.prompt_id,
                    version.version,
                    version.content,
                    version.change_summary,
                    version.created_at,
                ],
            )
            .map_err(db_err)?;
        Ok(())
    }
}

fn row_to_entry(row: &Row<'_>) -> rusqlite::Result<SyncLogEntry> {
    Ok(SyncLogEntry {
        id: row.get(0)?,
        entity_type: row.get(1)?,
        entity_id: row.get(2)?,
        operation: row.get(3)?,
        payload: row.get(4)?,
        hlc: row.get(5)?,
        device_id: row.get(6)?,
    })
}

impl SyncLogStore for SqliteSyncLogStore {
    fn record(
        &self,
        entity_type: &str,
        entity_id: &str,
        operation: &str,
        payload: &str,
    ) -> Result<(), AppError> {
        let id = Uuid::new_v4().to_string();
        let hlc = current_hlc();
        let now = now_timestamp();
        let conn = self.lock()?;
        conn.inner()
            .execute(
                "INSERT INTO sync_log (id, entity_type, entity_id, operation, payload, hlc, \
                 device_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    id,
                    entity_type,
                    entity_id,
                    operation,
                    payload,
                    hlc,
                    self.device_id,
                    now,
                ],
            )
            .map_err(db_err)?;
        Ok(())
    }

    fn pending(&self) -> Result<Vec<SyncLogEntry>, AppError> {
        let conn = self.lock()?;
        let mut stmt = conn
            .inner()
            .prepare(
                "SELECT id, entity_type, entity_id, operation, payload, hlc, device_id \
                 FROM sync_log ORDER BY rowid",
            )
            .map_err(db_err)?;
        let rows = stmt.query_map([], row_to_entry).map_err(db_err)?;
        rows.map(|r| r.map_err(db_err)).collect()
    }

    fn apply(&self, entries: &[SyncLogEntry]) -> Result<(), AppError> {
        for entry in entries {
            self.apply_one(entry)?;
        }
        Ok(())
    }

    fn mark_synced(&self, entries: &[SyncLogEntry]) -> Result<(), AppError> {
        if entries.is_empty() {
            return Ok(());
        }
        let mut conn = self.lock()?;
        let tx = conn.inner_mut().transaction().map_err(db_err)?;
        for e in entries {
            tx.execute("DELETE FROM sync_log WHERE id = ?1", params![e.id])
                .map_err(db_err)?;
        }
        tx.commit().map_err(db_err)?;
        Ok(())
    }
}

impl SyncConfigStore for SqliteSyncLogStore {
    fn load_config(&self) -> Result<Option<SyncConfig>, AppError> {
        let conn = self.lock()?;
        conn.inner()
            .query_row(
                "SELECT url, username, token, enabled FROM sync_config WHERE id = 1",
                [],
                |row| {
                    Ok(SyncConfig {
                        url: row.get(0)?,
                        username: row.get(1)?,
                        token: row.get(2)?,
                        enabled: row.get::<_, i64>(3)? != 0,
                    })
                },
            )
            .optional()
            .map_err(db_err)
    }

    fn save_config(&self, config: &SyncConfig) -> Result<(), AppError> {
        let enabled = i64::from(config.enabled);
        let conn = self.lock()?;
        conn.inner()
            .execute(
                "INSERT INTO sync_config (id, url, username, token, enabled) \
                 VALUES (1, ?1, ?2, ?3, ?4) \
                 ON CONFLICT(id) DO UPDATE SET \
                   url = excluded.url, \
                   username = excluded.username, \
                   token = excluded.token, \
                   enabled = excluded.enabled",
                params![config.url, config.username, config.token, enabled],
            )
            .map_err(db_err)?;
        Ok(())
    }
}
