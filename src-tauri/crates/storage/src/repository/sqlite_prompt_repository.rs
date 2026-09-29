use std::path::Path;
use std::sync::Mutex;

use prompt_core::{
    AppError, CreatePromptInput, Prompt, PromptQuery, PromptRepository, UpdatePromptInput,
};
use rusqlite::{params_from_iter, OptionalExtension, Row};
use uuid::Uuid;

use crate::sqlite::Connection;

/// 统一的列清单，保证所有查询回列顺序一致，供 `row_to_prompt` 按序读取。
const PROMPT_COLUMNS: &str = "id, title, content, description, category, \
     is_pinned, is_favorite, rating, created_at, updated_at, deleted_at";

/// SQLite 实现的提示词仓库（对应 M1-A2，明文 SQLite，rusqlite bundled）。
pub struct SqlitePromptRepository {
    conn: Mutex<Connection>,
}

impl SqlitePromptRepository {
    /// 打开内存数据库并完成迁移（单元测试用）。
    pub fn open_in_memory() -> Result<Self, AppError> {
        let mut conn = Connection::open_in_memory().map_err(db_err)?;
        conn.migrate().map_err(db_err)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 打开文件数据库并完成迁移。
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let mut conn = Connection::open(path).map_err(db_err)?;
        conn.migrate().map_err(db_err)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 由已打开的连接构造仓库（供装配阶段复用连接）。
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }
}

impl PromptRepository for SqlitePromptRepository {
    fn find_all(&self, query: &PromptQuery) -> Result<Vec<Prompt>, AppError> {
        let conn = self.lock()?;
        let inner = conn.inner();

        let mut sql = format!("SELECT {PROMPT_COLUMNS} FROM prompts WHERE deleted_at IS NULL");

        // M1 仅支持 keyword + limit；category / is_pinned / offset 留空实现。
        let mut args: Vec<String> = Vec::new();
        if let Some(keyword) = query
            .keyword
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            sql.push_str(" AND (title LIKE ?1 OR content LIKE ?1 OR description LIKE ?1)");
            args.push(format!("%{keyword}%"));
        }

        sql.push_str(" ORDER BY is_pinned DESC, updated_at DESC");
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }

        let mut stmt = inner.prepare(&sql).map_err(db_err)?;
        let rows = stmt
            .query_map(params_from_iter(args.iter()), row_to_prompt)
            .map_err(db_err)?;
        rows.map(|r| r.map_err(db_err)).collect()
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Prompt>, AppError> {
        let conn = self.lock()?;
        let sql =
            format!("SELECT {PROMPT_COLUMNS} FROM prompts WHERE id = ?1 AND deleted_at IS NULL");
        conn.inner()
            .query_row(&sql, [id], row_to_prompt)
            .optional()
            .map_err(db_err)
    }

    fn create(&self, input: &CreatePromptInput) -> Result<Prompt, AppError> {
        let now = now_timestamp();
        let prompt = Prompt {
            id: Uuid::new_v4().to_string(),
            title: input.title.clone(),
            content: input.content.clone(),
            description: input.description.clone(),
            category: input.category.clone(),
            is_pinned: false,
            is_favorite: false,
            rating: 0,
            created_at: now.clone(),
            updated_at: now,
            deleted_at: None,
        };

        let mut conn = self.lock()?;
        let tx = conn.inner_mut().transaction().map_err(db_err)?;
        tx.execute(
            "INSERT INTO prompts (id, title, content, description, category, \
             is_pinned, is_favorite, rating, created_at, updated_at, deleted_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                prompt.id,
                prompt.title,
                prompt.content,
                prompt.description,
                prompt.category,
                prompt.is_pinned,
                prompt.is_favorite,
                prompt.rating,
                prompt.created_at,
                prompt.updated_at,
                prompt.deleted_at,
            ],
        )
        .map_err(db_err)?;
        tx.execute(
            "INSERT INTO prompts_fts (id, title, content, description) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![prompt.id, prompt.title, prompt.content, prompt.description],
        )
        .map_err(db_err)?;
        tx.commit().map_err(db_err)?;

        Ok(prompt)
    }

    fn update(&self, id: &str, input: &UpdatePromptInput) -> Result<Prompt, AppError> {
        let mut conn = self.lock()?;

        let sql =
            format!("SELECT {PROMPT_COLUMNS} FROM prompts WHERE id = ?1 AND deleted_at IS NULL");
        let mut prompt = conn
            .inner()
            .query_row(&sql, [id], row_to_prompt)
            .optional()
            .map_err(db_err)?
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;

        // 字段级局部更新：Some 表示修改，None 表示保持不变。
        if let Some(v) = &input.title {
            prompt.title = v.clone();
        }
        if let Some(v) = &input.content {
            prompt.content = v.clone();
        }
        if let Some(v) = &input.description {
            prompt.description = Some(v.clone());
        }
        if let Some(v) = &input.category {
            prompt.category = Some(v.clone());
        }
        if let Some(v) = input.is_pinned {
            prompt.is_pinned = v;
        }
        if let Some(v) = input.is_favorite {
            prompt.is_favorite = v;
        }
        if let Some(v) = input.rating {
            prompt.rating = v;
        }
        prompt.updated_at = now_timestamp();

        let tx = conn.inner_mut().transaction().map_err(db_err)?;
        tx.execute(
            "UPDATE prompts SET title = ?2, content = ?3, description = ?4, category = ?5, \
             is_pinned = ?6, is_favorite = ?7, rating = ?8, updated_at = ?9 WHERE id = ?1",
            rusqlite::params![
                prompt.id,
                prompt.title,
                prompt.content,
                prompt.description,
                prompt.category,
                prompt.is_pinned,
                prompt.is_favorite,
                prompt.rating,
                prompt.updated_at,
            ],
        )
        .map_err(db_err)?;

        // FTS 同步：删除旧索引行，重写新索引行。
        let fts_row_id: Option<i64> = tx
            .query_row("SELECT rowid FROM prompts_fts WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .optional()
            .map_err(db_err)?;
        if let Some(rowid) = fts_row_id {
            tx.execute("DELETE FROM prompts_fts WHERE rowid = ?1", [rowid])
                .map_err(db_err)?;
        }
        tx.execute(
            "INSERT INTO prompts_fts (id, title, content, description) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![prompt.id, prompt.title, prompt.content, prompt.description],
        )
        .map_err(db_err)?;
        tx.commit().map_err(db_err)?;

        Ok(prompt)
    }

    fn delete(&self, id: &str) -> Result<(), AppError> {
        let mut conn = self.lock()?;
        let now = now_timestamp();

        let tx = conn.inner_mut().transaction().map_err(db_err)?;
        let affected = tx
            .execute(
                "UPDATE prompts SET deleted_at = ?2, updated_at = ?2 \
                 WHERE id = ?1 AND deleted_at IS NULL",
                rusqlite::params![id, now],
            )
            .map_err(db_err)?;
        if affected == 0 {
            return Err(AppError::NotFound(id.to_string()));
        }

        let fts_row_id: Option<i64> = tx
            .query_row("SELECT rowid FROM prompts_fts WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .optional()
            .map_err(db_err)?;
        if let Some(rowid) = fts_row_id {
            tx.execute("DELETE FROM prompts_fts WHERE rowid = ?1", [rowid])
                .map_err(db_err)?;
        }
        tx.commit().map_err(db_err)?;

        Ok(())
    }

    fn search(&self, keyword: &str) -> Result<Vec<Prompt>, AppError> {
        let conn = self.lock()?;
        let keyword = keyword.trim();
        if keyword.is_empty() {
            return Ok(Vec::new());
        }

        // trigram 分词器要求查询至少 3 个字符；更短的词退化为 LIKE 子串匹配。
        let sql = if keyword.chars().count() < 3 {
            format!(
                "SELECT {PROMPT_COLUMNS} FROM prompts
                 WHERE deleted_at IS NULL
                   AND (title LIKE ?1 OR content LIKE ?1 OR description LIKE ?1)
                 ORDER BY is_pinned DESC, updated_at DESC"
            )
        } else {
            format!(
                "SELECT {PROMPT_COLUMNS} FROM prompts
                 WHERE deleted_at IS NULL
                   AND id IN (SELECT id FROM prompts_fts WHERE prompts_fts MATCH ?1)
                 ORDER BY is_pinned DESC, updated_at DESC"
            )
        };

        let arg = if keyword.chars().count() < 3 {
            format!("%{keyword}%")
        } else {
            fts_match_query(keyword)
        };

        let mut stmt = conn.inner().prepare(&sql).map_err(db_err)?;
        let rows = stmt.query_map([arg], row_to_prompt).map_err(db_err)?;
        rows.map(|r| r.map_err(db_err)).collect()
    }
}

impl SqlitePromptRepository {
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, AppError> {
        self.conn
            .lock()
            .map_err(|_| AppError::Database("repository 连接锁已中毒".to_string()))
    }
}

/// 将 FTS5 查询词转义为带引号的短语，避免特殊字符破坏 MATCH 语法。
fn fts_match_query(keyword: &str) -> String {
    format!("\"{}\"", keyword.replace('"', "\"\""))
}

fn row_to_prompt(row: &Row<'_>) -> rusqlite::Result<Prompt> {
    Ok(Prompt {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        description: row.get(3)?,
        category: row.get(4)?,
        is_pinned: row.get::<_, i64>(5)? != 0,
        is_favorite: row.get::<_, i64>(6)? != 0,
        rating: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        deleted_at: row.get(10)?,
    })
}

fn now_timestamp() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn db_err(e: rusqlite::Error) -> AppError {
    AppError::Database(e.to_string())
}
