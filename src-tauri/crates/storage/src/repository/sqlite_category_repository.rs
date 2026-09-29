use std::path::Path;
use std::sync::MutexGuard;

use prompt_core::{
    AppError, Category, CategoryRepository, CreateCategoryInput, UpdateCategoryInput,
};
use rusqlite::{OptionalExtension, Row};
use uuid::Uuid;

use crate::sqlite::{Connection, Db};
use crate::util::{db_err, now_timestamp};

const CATEGORY_COLUMNS: &str = "id, name, parent_id, sort_order, created_at";

/// SQLite 实现的分类仓库（M2 组织层，树形分类）。
pub struct SqliteCategoryRepository {
    db: Db,
}

impl SqliteCategoryRepository {
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

impl CategoryRepository for SqliteCategoryRepository {
    fn find_all(&self) -> Result<Vec<Category>, AppError> {
        let conn = self.lock()?;
        let sql = format!(
            "SELECT {CATEGORY_COLUMNS} FROM categories ORDER BY sort_order ASC, created_at ASC"
        );
        let mut stmt = conn.inner().prepare(&sql).map_err(db_err)?;
        let rows = stmt.query_map([], row_to_category).map_err(db_err)?;
        rows.map(|r| r.map_err(db_err)).collect()
    }

    fn create(&self, input: &CreateCategoryInput) -> Result<Category, AppError> {
        let category = Category {
            id: Uuid::new_v4().to_string(),
            name: input.name.clone(),
            parent_id: input.parent_id.clone(),
            sort_order: input.sort_order.unwrap_or(0),
            created_at: now_timestamp(),
        };

        let conn = self.lock()?;
        conn.inner()
            .execute(
                "INSERT INTO categories (id, name, parent_id, sort_order, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    category.id,
                    category.name,
                    category.parent_id,
                    category.sort_order,
                    category.created_at,
                ],
            )
            .map_err(db_err)?;

        Ok(category)
    }

    fn update(&self, id: &str, input: &UpdateCategoryInput) -> Result<Category, AppError> {
        let conn = self.lock()?;

        let sql = format!("SELECT {CATEGORY_COLUMNS} FROM categories WHERE id = ?1");
        let mut category = conn
            .inner()
            .query_row(&sql, [id], row_to_category)
            .optional()
            .map_err(db_err)?
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;

        if let Some(v) = &input.name {
            category.name = v.clone();
        }
        if let Some(v) = &input.parent_id {
            category.parent_id = Some(v.clone());
        }
        if let Some(v) = input.sort_order {
            category.sort_order = v;
        }

        conn.inner()
            .execute(
                "UPDATE categories SET name = ?2, parent_id = ?3, sort_order = ?4 WHERE id = ?1",
                rusqlite::params![
                    category.id,
                    category.name,
                    category.parent_id,
                    category.sort_order
                ],
            )
            .map_err(db_err)?;

        Ok(category)
    }

    fn delete(&self, id: &str) -> Result<(), AppError> {
        let mut conn = self.lock()?;
        let tx = conn.inner_mut().transaction().map_err(db_err)?;
        let affected = tx
            .execute("DELETE FROM categories WHERE id = ?1", [id])
            .map_err(db_err)?;
        if affected == 0 {
            return Err(AppError::NotFound(id.to_string()));
        }
        // 解除提示词对该分类的引用，避免悬挂外键。
        tx.execute(
            "UPDATE prompts SET category_id = NULL WHERE category_id = ?1",
            [id],
        )
        .map_err(db_err)?;
        tx.commit().map_err(db_err)?;
        Ok(())
    }
}

fn row_to_category(row: &Row<'_>) -> rusqlite::Result<Category> {
    Ok(Category {
        id: row.get(0)?,
        name: row.get(1)?,
        parent_id: row.get(2)?,
        sort_order: row.get(3)?,
        created_at: row.get(4)?,
    })
}
