use std::path::Path;
use std::sync::MutexGuard;

use prompt_core::{AppError, CreateTagInput, Tag, TagRepository, UpdateTagInput};
use rusqlite::{OptionalExtension, Row};
use uuid::Uuid;

use crate::sqlite::{Connection, Db};
use crate::util::{db_err, now_timestamp};

const TAG_COLUMNS: &str = "id, name, parent_id, color, created_at";

/// SQLite 实现的标签仓库（M2 组织层，层级标签）。
pub struct SqliteTagRepository {
    db: Db,
}

impl SqliteTagRepository {
    pub fn open_in_memory() -> Result<Self, AppError> {
        Ok(Self::new(Db::open_in_memory()?))
    }

    pub fn open(path: &Path) -> Result<Self, AppError> {
        Ok(Self::new(Db::open(path)?))
    }

    pub fn new(db: Db) -> Self {
        Self { db }
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        self.db.lock()
    }
}

impl TagRepository for SqliteTagRepository {
    fn find_all(&self) -> Result<Vec<Tag>, AppError> {
        let conn = self.lock()?;
        let sql = format!("SELECT {TAG_COLUMNS} FROM tags ORDER BY created_at ASC");
        let mut stmt = conn.inner().prepare(&sql).map_err(db_err)?;
        let rows = stmt.query_map([], row_to_tag).map_err(db_err)?;
        rows.map(|r| r.map_err(db_err)).collect()
    }

    fn create(&self, input: &CreateTagInput) -> Result<Tag, AppError> {
        let tag = Tag {
            id: Uuid::new_v4().to_string(),
            name: input.name.clone(),
            parent_id: input.parent_id.clone(),
            color: input.color.clone(),
            created_at: now_timestamp(),
        };

        let conn = self.lock()?;
        conn.inner()
            .execute(
                "INSERT INTO tags (id, name, parent_id, color, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![tag.id, tag.name, tag.parent_id, tag.color, tag.created_at],
            )
            .map_err(db_err)?;

        Ok(tag)
    }

    fn update(&self, id: &str, input: &UpdateTagInput) -> Result<Tag, AppError> {
        let conn = self.lock()?;

        let sql = format!("SELECT {TAG_COLUMNS} FROM tags WHERE id = ?1");
        let mut tag = conn
            .inner()
            .query_row(&sql, [id], row_to_tag)
            .optional()
            .map_err(db_err)?
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;

        if let Some(v) = &input.name {
            tag.name = v.clone();
        }
        if let Some(v) = &input.parent_id {
            tag.parent_id = Some(v.clone());
        }
        if let Some(v) = &input.color {
            tag.color = Some(v.clone());
        }

        conn.inner()
            .execute(
                "UPDATE tags SET name = ?2, parent_id = ?3, color = ?4 WHERE id = ?1",
                rusqlite::params![tag.id, tag.name, tag.parent_id, tag.color],
            )
            .map_err(db_err)?;

        Ok(tag)
    }

    fn delete(&self, id: &str) -> Result<(), AppError> {
        let mut conn = self.lock()?;
        let tx = conn.inner_mut().transaction().map_err(db_err)?;
        let affected = tx
            .execute("DELETE FROM tags WHERE id = ?1", [id])
            .map_err(db_err)?;
        if affected == 0 {
            return Err(AppError::NotFound(id.to_string()));
        }
        // 清理该标签的提示词关联，避免悬挂引用。
        tx.execute("DELETE FROM prompt_tags WHERE tag_id = ?1", [id])
            .map_err(db_err)?;
        tx.commit().map_err(db_err)?;
        Ok(())
    }
}

fn row_to_tag(row: &Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        parent_id: row.get(2)?,
        color: row.get(3)?,
        created_at: row.get(4)?,
    })
}
