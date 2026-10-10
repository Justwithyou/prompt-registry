//! 数据备份 / 迁移：全量导出与导入（对应《个人提示词管理平台导入导出设计方案》）。
//!
//! 与「CLI/Skills 输出」（`crates/export`，渲染成 SKILL.md / md / json）不同，本模块
//! 面向「从一个用户导出、导入到另一个用户（设备/账号）」的可移植交接场景，完整保留
//! 分类 / 标签 / 项目 / 提示词 / 关联 / 历史版本的原始 ID 与层级关系，做到可无损往返。
//!
//! 所有读写均直接基于共享 [`Db`]，跨越多个业务表在单个事务内完成，保证原子性。

use prompt_core::{AppError, Category, Project, Prompt, PromptVersion, Tag};
use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

use crate::repository::sqlite_prompt_repository::{row_to_prompt, PROMPT_COLUMNS};
use crate::sqlite::Db;
use crate::util::{db_err, now_timestamp};

/// 导出文件格式版本；仅当格式发生不兼容演进时递增。
pub const SCHEMA_VERSION: u32 = 1;

/// 平台标识，用于导入时校验来源一致性。
pub const PLATFORM: &str = "prompt-manager";

// ===== 导出文件结构 =====

/// 导出文件顶层结构（写为单个 JSON 文件）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataBundle {
    pub schema_version: u32,
    pub platform: String,
    pub exported_at: String,
    pub data: BundleData,
}

/// 全量业务数据（按实体分组）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleData {
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
    pub projects: Vec<Project>,
    pub prompts: Vec<Prompt>,
    pub prompt_projects: Vec<PromptProjectLink>,
    pub prompt_tags: Vec<PromptTagLink>,
    pub versions: Vec<PromptVersion>,
}

/// 提示词-项目关联。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptProjectLink {
    pub prompt_id: String,
    pub project_id: String,
}

/// 提示词-标签关联。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptTagLink {
    pub prompt_id: String,
    pub tag_id: String,
}

// ===== 导入报告 =====

/// 单个实体类型的导入统计。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportCount {
    pub imported: usize,
    pub skipped: usize,
}

impl ImportCount {
    /// 依据「受影响行数」累计：>0 视为新增，否则为跳过已有。
    fn add(&mut self, affected: usize) {
        if affected > 0 {
            self.imported += 1;
        } else {
            self.skipped += 1;
        }
    }
}

/// 一次导入的汇总结果（按实体类型拆分）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub categories: ImportCount,
    pub tags: ImportCount,
    pub projects: ImportCount,
    pub prompts: ImportCount,
    pub versions: ImportCount,
}

// ===== 导出 =====

/// 从 [`Db`] 读取全量业务数据，构造导出包（内存态，不落盘）。
pub fn export_bundle(db: &Db) -> Result<DataBundle, AppError> {
    let conn = db.lock()?;
    let inner = conn.inner();

    let categories = query_all(
        inner,
        "SELECT id, name, parent_id, sort_order, created_at \
         FROM categories ORDER BY sort_order ASC, created_at ASC",
        row_to_category,
    )?;
    let tags = query_all(
        inner,
        "SELECT id, name, parent_id, color, created_at \
         FROM tags ORDER BY created_at ASC",
        row_to_tag,
    )?;
    let projects = query_all(
        inner,
        "SELECT id, name, description, color, created_at, updated_at \
         FROM projects ORDER BY created_at ASC",
        row_to_project,
    )?;
    let prompts = query_all(
        inner,
        &format!(
            "SELECT {PROMPT_COLUMNS} FROM prompts WHERE deleted_at IS NULL \
                  ORDER BY created_at ASC"
        ),
        row_to_prompt,
    )?;

    let prompt_projects = query_all(
        inner,
        "SELECT prompt_id, project_id FROM prompt_projects",
        |row| {
            Ok(PromptProjectLink {
                prompt_id: row.get(0)?,
                project_id: row.get(1)?,
            })
        },
    )?;
    let prompt_tags = query_all(inner, "SELECT prompt_id, tag_id FROM prompt_tags", |row| {
        Ok(PromptTagLink {
            prompt_id: row.get(0)?,
            tag_id: row.get(1)?,
        })
    })?;
    // 仅导出「所属提示词未被删除」的版本，避免导入后产生悬挂引用。
    let versions = query_all(
        inner,
        "SELECT id, prompt_id, version, content, change_summary, created_at \
         FROM prompt_versions WHERE prompt_id IN (SELECT id FROM prompts WHERE deleted_at IS NULL) \
         ORDER BY created_at ASC, rowid ASC",
        row_to_version,
    )?;

    Ok(DataBundle {
        schema_version: SCHEMA_VERSION,
        platform: PLATFORM.to_string(),
        exported_at: now_timestamp(),
        data: BundleData {
            categories,
            tags,
            projects,
            prompts,
            prompt_projects,
            prompt_tags,
            versions,
        },
    })
}

// ===== 导入 =====

/// 将数据包写入 [`Db`]；`overwrite` 为真时先清空业务数据再导入，否则追加合并。
///
/// 全程在单个事务内完成；任意一步失败即整体回滚。ID 冲突一律「跳过已有」
/// （`INSERT OR IGNORE`），不覆盖目标库本地记录。
pub fn import_bundle(
    db: &Db,
    bundle: &DataBundle,
    overwrite: bool,
) -> Result<ImportReport, AppError> {
    let mut conn = db.lock()?;
    let tx = conn.inner_mut().transaction().map_err(db_err)?;

    if overwrite {
        tx.execute_batch(
            "DELETE FROM prompt_versions;
             DELETE FROM prompt_tags;
             DELETE FROM prompt_projects;
             DELETE FROM prompts;
             DELETE FROM tags;
             DELETE FROM projects;
             DELETE FROM categories;
             INSERT INTO prompts_fts(prompts_fts) VALUES('delete-all');",
        )
        .map_err(db_err)?;
    }

    let mut report = ImportReport::default();
    let data = &bundle.data;

    for c in &data.categories {
        let n = tx
            .execute(
                "INSERT OR IGNORE INTO categories (id, name, parent_id, sort_order, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![c.id, c.name, c.parent_id, c.sort_order, c.created_at],
            )
            .map_err(db_err)?;
        report.categories.add(n);
    }

    for t in &data.tags {
        let n = tx
            .execute(
                "INSERT OR IGNORE INTO tags (id, name, parent_id, color, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![t.id, t.name, t.parent_id, t.color, t.created_at],
            )
            .map_err(db_err)?;
        report.tags.add(n);
    }

    for p in &data.projects {
        let n = tx
            .execute(
                "INSERT OR IGNORE INTO projects (id, name, description, color, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![p.id, p.name, p.description, p.color, p.created_at, p.updated_at],
            )
            .map_err(db_err)?;
        report.projects.add(n);
    }

    for p in &data.prompts {
        let n = tx
            .execute(
                "INSERT OR IGNORE INTO prompts (id, title, content, description, category, \
                 category_id, is_pinned, is_favorite, rating, created_at, updated_at, deleted_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    p.id,
                    p.title,
                    p.content,
                    p.description,
                    p.category,
                    p.category_id,
                    p.is_pinned as i64,
                    p.is_favorite as i64,
                    p.rating,
                    p.created_at,
                    p.updated_at,
                    p.deleted_at,
                ],
            )
            .map_err(db_err)?;
        report.prompts.add(n);
        if n > 0 {
            // 仅新插入的提示词写入全文索引（跳过已有时不重复索引）。
            crate::repository::sqlite_prompt_repository::sync_fts(&tx, p)?;
        }
    }

    for link in &data.prompt_projects {
        tx.execute(
            "INSERT OR IGNORE INTO prompt_projects (prompt_id, project_id) VALUES (?1, ?2)",
            params![link.prompt_id, link.project_id],
        )
        .map_err(db_err)?;
    }

    for link in &data.prompt_tags {
        tx.execute(
            "INSERT OR IGNORE INTO prompt_tags (prompt_id, tag_id) VALUES (?1, ?2)",
            params![link.prompt_id, link.tag_id],
        )
        .map_err(db_err)?;
    }

    for v in &data.versions {
        let n = tx
            .execute(
                "INSERT OR IGNORE INTO prompt_versions (id, prompt_id, version, content, \
                 change_summary, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    v.id,
                    v.prompt_id,
                    v.version,
                    v.content,
                    v.change_summary,
                    v.created_at
                ],
            )
            .map_err(db_err)?;
        report.versions.add(n);
    }

    tx.commit().map_err(db_err)?;
    Ok(report)
}

/// 校验数据包来源与版本。非法时返回错误，由命令层拒绝导入。
pub fn validate(bundle: &DataBundle) -> Result<(), AppError> {
    if bundle.platform != PLATFORM {
        return Err(AppError::Io("导入文件不是本平台导出的数据".into()));
    }
    if bundle.schema_version != SCHEMA_VERSION {
        return Err(AppError::Io(format!(
            "不支持的导出文件版本: {}（当前支持 {}）",
            bundle.schema_version, SCHEMA_VERSION
        )));
    }
    Ok(())
}

/// 序列化数据包为 JSON 文本（供写文件）。
pub fn bundle_to_json(bundle: &DataBundle) -> Result<String, AppError> {
    serde_json::to_string_pretty(bundle).map_err(io_err)
}

/// 从 JSON 文本反序列化数据包（供读文件）。
pub fn bundle_from_json(text: &str) -> Result<DataBundle, AppError> {
    serde_json::from_str(text).map_err(io_err)
}

// ===== 内部工具 =====

fn io_err(e: impl std::fmt::Display) -> AppError {
    AppError::Io(e.to_string())
}

fn query_all<T, F>(inner: &rusqlite::Connection, sql: &str, map: F) -> Result<Vec<T>, AppError>
where
    F: Fn(&Row<'_>) -> rusqlite::Result<T>,
{
    let mut stmt = inner.prepare(sql).map_err(db_err)?;
    let rows = stmt.query_map([], map).map_err(db_err)?;
    rows.map(|r| r.map_err(db_err)).collect()
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

fn row_to_tag(row: &Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        parent_id: row.get(2)?,
        color: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn row_to_project(row: &Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        color: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{
        SqliteCategoryRepository, SqliteProjectRepository, SqlitePromptRepository,
        SqliteTagRepository,
    };
    use prompt_core::{
        CategoryRepository, CreateCategoryInput, CreateProjectInput, CreatePromptInput,
        CreateTagInput, ProjectRepository, PromptRepository, TagRepository,
    };

    /// 在内存库中写入一组覆盖分类/标签/项目/提示词的最小数据。
    fn seed(db: &Db) {
        let cat = SqliteCategoryRepository::new(db.clone());
        let tag = SqliteTagRepository::new(db.clone());
        let proj = SqliteProjectRepository::new(db.clone());
        let prompt = SqlitePromptRepository::new(db.clone());

        let c = cat
            .create(&CreateCategoryInput {
                name: "工作".into(),
                ..Default::default()
            })
            .unwrap();
        let t = tag
            .create(&CreateTagInput {
                name: "模板".into(),
                ..Default::default()
            })
            .unwrap();
        let p = proj
            .create(&CreateProjectInput {
                name: "内部流程".into(),
                ..Default::default()
            })
            .unwrap();
        let created = prompt
            .create(&CreatePromptInput {
                title: "周报".into(),
                content: "内容".into(),
                ..Default::default()
            })
            .unwrap();
        prompt
            .set_relations(
                &created.id,
                &prompt_core::PromptRelationsInput {
                    project_ids: Some(vec![p.id]),
                    tag_ids: Some(vec![t.id]),
                    category_id: Some(c.id),
                    ..Default::default()
                },
            )
            .unwrap();
    }

    #[test]
    fn json_roundtrip_preserves_bundle() {
        let db = Db::open_in_memory().unwrap();
        seed(&db);
        let bundle = export_bundle(&db).unwrap();

        let json = bundle_to_json(&bundle).unwrap();
        let restored = bundle_from_json(&json).unwrap();
        validate(&restored).unwrap();

        assert_eq!(restored.data.categories.len(), bundle.data.categories.len());
        assert_eq!(restored.data.prompts.len(), bundle.data.prompts.len());
        assert_eq!(restored.data.prompts[0].id, bundle.data.prompts[0].id);
    }

    #[test]
    fn import_into_fresh_db_restores_entities() {
        let src = Db::open_in_memory().unwrap();
        seed(&src);
        let bundle = export_bundle(&src).unwrap();

        let dst = Db::open_in_memory().unwrap();
        let report = import_bundle(&dst, &bundle, false).unwrap();

        assert_eq!(report.categories.imported, 1);
        assert_eq!(report.tags.imported, 1);
        assert_eq!(report.projects.imported, 1);
        assert_eq!(report.prompts.imported, 1);
        assert!(report.prompts.skipped == 0);

        // 验证可读回并保留关系引用。
        let prompt_repo = SqlitePromptRepository::new(dst.clone());
        let all = prompt_repo
            .find_all(&prompt_core::PromptQuery::default())
            .unwrap();
        assert_eq!(all.len(), 1);
        let rel = prompt_repo.get_relations(&all[0].id).unwrap();
        assert_eq!(rel.project_ids.len(), 1);
        assert_eq!(rel.tag_ids.len(), 1);
        assert!(rel.category_id.is_some());
    }

    #[test]
    fn reimport_skips_existing_ids() {
        let src = Db::open_in_memory().unwrap();
        seed(&src);
        let bundle = export_bundle(&src).unwrap();

        let dst = Db::open_in_memory().unwrap();
        import_bundle(&dst, &bundle, false).unwrap();
        let second = import_bundle(&dst, &bundle, false).unwrap();

        assert_eq!(second.prompts.skipped, 1);
        assert_eq!(second.prompts.imported, 0);
        let prompt_repo = SqlitePromptRepository::new(dst.clone());
        assert_eq!(
            prompt_repo
                .find_all(&prompt_core::PromptQuery::default())
                .unwrap()
                .len(),
            1
        );
    }
}
