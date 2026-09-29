use std::path::Path;
use std::sync::MutexGuard;

use prompt_core::{
    AppError, BatchUpdateInput, CreatePromptInput, Prompt, PromptQuery, PromptRelationsInput,
    PromptRepository, PromptSortBy, UpdatePromptInput,
};
use rusqlite::{params_from_iter, OptionalExtension, Row, Transaction};
use uuid::Uuid;

use crate::sqlite::Db;
use crate::util::{db_err, latest_version, next_version, now_timestamp};

/// 统一的列清单，保证所有查询回列顺序一致，供 `row_to_prompt` 按序读取。
pub(crate) const PROMPT_COLUMNS: &str = "id, title, content, description, category, category_id, \
     is_pinned, is_favorite, rating, created_at, updated_at, deleted_at";

/// SQLite 实现的提示词仓库（对应 M1-A2，明文 SQLite，rusqlite bundled）。
///
/// M2 起持有可共享的 [`Db`]，与项目 / 标签 / 分类仓库共享同一连接。
pub struct SqlitePromptRepository {
    db: Db,
}

impl SqlitePromptRepository {
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

impl PromptRepository for SqlitePromptRepository {
    fn find_all(&self, query: &PromptQuery) -> Result<Vec<Prompt>, AppError> {
        let conn = self.lock()?;
        let inner = conn.inner();

        let mut sql = format!("SELECT {PROMPT_COLUMNS} FROM prompts WHERE deleted_at IS NULL");
        let mut args: Vec<String> = Vec::new();

        if let Some(keyword) = query
            .keyword
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            sql.push_str(" AND (title LIKE ? OR content LIKE ? OR description LIKE ?)");
            let like = format!("%{keyword}%");
            args.push(like.clone());
            args.push(like.clone());
            args.push(like);
        }

        if let Some(category_id) = query.category_id.as_deref().filter(|s| !s.is_empty()) {
            sql.push_str(" AND category_id = ?");
            args.push(category_id.to_string());
        }

        if let Some(project_id) = query.project_id.as_deref().filter(|s| !s.is_empty()) {
            sql.push_str(" AND id IN (SELECT prompt_id FROM prompt_projects WHERE project_id = ?)");
            args.push(project_id.to_string());
        }

        if let Some(tag_ids) = query.tag_ids.as_ref().filter(|v| !v.is_empty()) {
            let placeholders = tag_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            sql.push_str(&format!(
                " AND id IN (SELECT prompt_id FROM prompt_tags WHERE tag_id IN ({placeholders}) \
                 GROUP BY prompt_id HAVING COUNT(DISTINCT tag_id) = {})",
                tag_ids.len()
            ));
            args.extend(tag_ids.iter().cloned());
        }

        // 布尔条件（收藏 / 置顶）为受信 Rust bool，直接内联为字面量，避免类型绑定问题。
        if let Some(is_favorite) = query.is_favorite {
            sql.push_str(if is_favorite {
                " AND is_favorite = 1"
            } else {
                " AND is_favorite = 0"
            });
        }
        if let Some(is_pinned) = query.is_pinned {
            sql.push_str(if is_pinned {
                " AND is_pinned = 1"
            } else {
                " AND is_pinned = 0"
            });
        }

        // 排序：置顶恒优先，再按 sort_by 倒序。
        sql.push_str(" ORDER BY is_pinned DESC");
        match query.sort_by.unwrap_or(PromptSortBy::UpdatedAt) {
            PromptSortBy::UpdatedAt => sql.push_str(", updated_at DESC"),
            PromptSortBy::Rating => sql.push_str(", rating DESC, updated_at DESC"),
            PromptSortBy::CreatedAt => sql.push_str(", created_at DESC"),
        }

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
            category_id: None,
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
            "INSERT INTO prompts (id, title, content, description, category, category_id, \
             is_pinned, is_favorite, rating, created_at, updated_at, deleted_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
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

        // M3-A5：在应用变更前捕获旧正文，并为本次保存生成历史版本（旧内容 + 递增版本号）。
        let old_content = prompt.content.clone();
        let change_summary = if input.content.is_some() {
            "更新内容"
        } else if input.title.is_some() {
            "更新标题"
        } else if input.description.is_some() {
            "更新描述"
        } else {
            "更新提示词"
        };
        let version = next_version(latest_version(conn.inner(), id)?.as_deref());
        let version_id = Uuid::new_v4().to_string();
        let now = now_timestamp();

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
        prompt.updated_at = now;

        let tx = conn.inner_mut().transaction().map_err(db_err)?;
        // 注意：category_id 由 set_relations 管理，update 不覆盖。
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

        // 追加历史版本：记录本次保存前的内容，语义化版本号递增。
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

        delete_fts(&tx, id)?;
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

    fn set_relations(&self, id: &str, input: &PromptRelationsInput) -> Result<Prompt, AppError> {
        let mut conn = self.lock()?;

        let sql =
            format!("SELECT {PROMPT_COLUMNS} FROM prompts WHERE id = ?1 AND deleted_at IS NULL");
        let mut prompt = conn
            .inner()
            .query_row(&sql, [id], row_to_prompt)
            .optional()
            .map_err(db_err)?
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;

        if let Some(v) = input.is_pinned {
            prompt.is_pinned = v;
        }
        if let Some(v) = input.is_favorite {
            prompt.is_favorite = v;
        }
        if let Some(v) = input.rating {
            prompt.rating = v;
        }
        if let Some(v) = &input.category_id {
            prompt.category_id = Some(v.clone());
        }
        prompt.updated_at = now_timestamp();

        let tx = conn.inner_mut().transaction().map_err(db_err)?;

        tx.execute(
            "UPDATE prompts SET category_id = ?2, is_pinned = ?3, is_favorite = ?4, \
             rating = ?5, updated_at = ?6 WHERE id = ?1",
            rusqlite::params![
                prompt.id,
                prompt.category_id,
                prompt.is_pinned,
                prompt.is_favorite,
                prompt.rating,
                prompt.updated_at,
            ],
        )
        .map_err(db_err)?;

        // 项目关联：整组替换。
        if let Some(project_ids) = &input.project_ids {
            tx.execute("DELETE FROM prompt_projects WHERE prompt_id = ?1", [id])
                .map_err(db_err)?;
            for project_id in project_ids {
                tx.execute(
                    "INSERT OR IGNORE INTO prompt_projects (prompt_id, project_id) VALUES (?1, ?2)",
                    rusqlite::params![id, project_id],
                )
                .map_err(db_err)?;
            }
        }

        // 标签关联：整组替换。
        if let Some(tag_ids) = &input.tag_ids {
            tx.execute("DELETE FROM prompt_tags WHERE prompt_id = ?1", [id])
                .map_err(db_err)?;
            for tag_id in tag_ids {
                tx.execute(
                    "INSERT OR IGNORE INTO prompt_tags (prompt_id, tag_id) VALUES (?1, ?2)",
                    rusqlite::params![id, tag_id],
                )
                .map_err(db_err)?;
            }
        }

        tx.commit().map_err(db_err)?;

        Ok(prompt)
    }

    fn batch_update(&self, ids: &[String], batch: &BatchUpdateInput) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }

        let mut conn = self.lock()?;
        let tx = conn.inner_mut().transaction().map_err(db_err)?;

        if batch.delete {
            let now = now_timestamp();
            for id in ids {
                tx.execute(
                    "UPDATE prompts SET deleted_at = ?2, updated_at = ?2 \
                     WHERE id = ?1 AND deleted_at IS NULL",
                    rusqlite::params![id, now],
                )
                .map_err(db_err)?;
                delete_fts(&tx, id)?;
            }
            tx.commit().map_err(db_err)?;
            return Ok(());
        }

        if let Some(add_tag_ids) = &batch.add_tag_ids {
            for id in ids {
                for tag_id in add_tag_ids {
                    tx.execute(
                        "INSERT OR IGNORE INTO prompt_tags (prompt_id, tag_id) VALUES (?1, ?2)",
                        rusqlite::params![id, tag_id],
                    )
                    .map_err(db_err)?;
                }
            }
        }

        if let Some(remove_tag_ids) = &batch.remove_tag_ids {
            for id in ids {
                for tag_id in remove_tag_ids {
                    tx.execute(
                        "DELETE FROM prompt_tags WHERE prompt_id = ?1 AND tag_id = ?2",
                        rusqlite::params![id, tag_id],
                    )
                    .map_err(db_err)?;
                }
            }
        }

        // 「移动项目」：整组替换为单一项目（与 set_relations 的 project_ids 语义一致）。
        if let Some(project_id) = &batch.set_project_id {
            for id in ids {
                tx.execute("DELETE FROM prompt_projects WHERE prompt_id = ?1", [id])
                    .map_err(db_err)?;
                tx.execute(
                    "INSERT INTO prompt_projects (prompt_id, project_id) VALUES (?1, ?2)",
                    rusqlite::params![id, project_id],
                )
                .map_err(db_err)?;
            }
        }

        tx.commit().map_err(db_err)?;
        Ok(())
    }
}

/// 将 FTS5 查询词转义为带引号的短语，避免特殊字符破坏 MATCH 语法。
fn fts_match_query(keyword: &str) -> String {
    format!("\"{}\"", keyword.replace('"', "\"\""))
}

/// 删除某条提示词在 FTS 索引中的行（若存在），供本仓库与版本仓库复用。
pub(crate) fn delete_fts(tx: &Transaction<'_>, id: &str) -> Result<(), AppError> {
    let rowid: Option<i64> = tx
        .query_row("SELECT rowid FROM prompts_fts WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .optional()
        .map_err(db_err)?;
    if let Some(rowid) = rowid {
        tx.execute("DELETE FROM prompts_fts WHERE rowid = ?1", [rowid])
            .map_err(db_err)?;
    }
    Ok(())
}

/// 将某条提示词的最新字段同步进 FTS 索引（先删后插）。
pub(crate) fn sync_fts(tx: &Transaction<'_>, prompt: &Prompt) -> Result<(), AppError> {
    delete_fts(tx, &prompt.id)?;
    tx.execute(
        "INSERT INTO prompts_fts (id, title, content, description) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![prompt.id, prompt.title, prompt.content, prompt.description],
    )
    .map_err(db_err)?;
    Ok(())
}

pub(crate) fn row_to_prompt(row: &Row<'_>) -> rusqlite::Result<Prompt> {
    Ok(Prompt {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        description: row.get(3)?,
        category: row.get(4)?,
        category_id: row.get(5)?,
        is_pinned: row.get::<_, i64>(6)? != 0,
        is_favorite: row.get::<_, i64>(7)? != 0,
        rating: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
        deleted_at: row.get(11)?,
    })
}
