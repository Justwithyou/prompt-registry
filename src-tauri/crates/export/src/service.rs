//! 导出内核服务：读数据 → 拼装 IR →（可选）渲染 → 序列化 → 写文件。

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use prompt_core::{
    AppError, Category, CategoryRepository, ProjectRepository, Prompt, PromptQuery,
    PromptRepository, Tag, TagRepository, TemplateExtractor, VariableDefinition,
};

use crate::item::{ExportFormat, ExportItem, ExportMeta, ExportSelector};
use crate::writer;

/// 已解析的组织名（分类 / 标签原文 + 项目名映射），供拼装 IR 时展开层级显示名。
struct Names {
    categories: Vec<Category>,
    tags: Vec<Tag>,
    project_names: HashMap<String, String>,
}

/// 单一导出内核：`cli` 与 `ipc` 复用，避免导出逻辑分叉。
pub struct ExportService {
    prompt_repo: Arc<dyn PromptRepository>,
    category_repo: Arc<dyn CategoryRepository>,
    tag_repo: Arc<dyn TagRepository>,
    project_repo: Arc<dyn ProjectRepository>,
    extractor: Arc<dyn TemplateExtractor>,
}

impl ExportService {
    pub fn new(
        prompt_repo: Arc<dyn PromptRepository>,
        category_repo: Arc<dyn CategoryRepository>,
        tag_repo: Arc<dyn TagRepository>,
        project_repo: Arc<dyn ProjectRepository>,
        extractor: Arc<dyn TemplateExtractor>,
    ) -> Self {
        Self {
            prompt_repo,
            category_repo,
            tag_repo,
            project_repo,
            extractor,
        }
    }

    /// 按选择器读取提示词并拼装为 IR 列表。
    pub fn items(
        &self,
        selector: &ExportSelector,
        keyword: Option<&str>,
        render: bool,
        vars: &HashMap<String, String>,
    ) -> Result<Vec<ExportItem>, AppError> {
        let prompts = self.load_prompts(selector, keyword)?;
        let names = self.resolve_names()?;
        let mut out = Vec::with_capacity(prompts.len());
        for p in prompts {
            out.push(self.build_item(&p, &names, render, vars)?);
        }
        Ok(out)
    }

    /// 渲染单条成品（CLI `run` / IPC `render_prompt`）。
    pub fn render_one(&self, id: &str, vars: &HashMap<String, String>) -> Result<String, AppError> {
        let prompt = self
            .prompt_repo
            .find_by_id(id)?
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;
        let definitions = self.extractor.extract_variables(&prompt.content);
        Ok(template::render(
            &prompt.content,
            &merge_defaults(&definitions, vars),
        ))
    }

    /// 导出到目录，返回写到磁盘的文件路径列表。
    pub fn export(
        &self,
        selector: &ExportSelector,
        format: ExportFormat,
        render: bool,
        vars: &HashMap<String, String>,
        out_dir: &Path,
    ) -> Result<Vec<String>, AppError> {
        let items = self.items(selector, None, render, vars)?;
        std::fs::create_dir_all(out_dir).map_err(io_err)?;

        let mut written = Vec::new();
        let mut seen: HashMap<String, usize> = HashMap::new();
        for item in &items {
            let slug = unique_slug(writer::slugify(&item.title), &mut seen);
            match format {
                ExportFormat::Skill => {
                    let dir = out_dir.join(&slug);
                    std::fs::create_dir_all(&dir).map_err(io_err)?;
                    let file = dir.join("SKILL.md");
                    std::fs::write(&file, writer::serialize(item, format)).map_err(io_err)?;
                    written.push(file.to_string_lossy().into_owned());
                }
                _ => {
                    let ext = match format {
                        ExportFormat::Markdown => "md",
                        ExportFormat::Json => "json",
                        ExportFormat::Skill => unreachable!(),
                    };
                    let file = out_dir.join(format!("{slug}.{ext}"));
                    std::fs::write(&file, writer::serialize(item, format)).map_err(io_err)?;
                    written.push(file.to_string_lossy().into_owned());
                }
            }
        }
        Ok(written)
    }

    fn load_prompts(
        &self,
        selector: &ExportSelector,
        keyword: Option<&str>,
    ) -> Result<Vec<Prompt>, AppError> {
        match selector {
            ExportSelector::Ids(ids) => ids
                .iter()
                .map(|id| {
                    self.prompt_repo
                        .find_by_id(id)?
                        .ok_or_else(|| AppError::NotFound(id.clone()))
                })
                .collect(),
            _ => {
                let mut query = PromptQuery {
                    keyword: keyword.map(str::to_string),
                    ..Default::default()
                };
                match selector {
                    ExportSelector::All => {}
                    ExportSelector::Categories(ids) => query.category_ids = Some(ids.clone()),
                    ExportSelector::Projects(ids) => query.project_ids = Some(ids.clone()),
                    ExportSelector::Tags(ids) => {
                        query.tag_ids = Some(ids.clone());
                        // 导出按「任一标签命中」并集，而非筛选面板的交集语义。
                        query.tag_match_any = true;
                    }
                    ExportSelector::Ids(_) => unreachable!(),
                }
                self.prompt_repo.find_all(&query)
            }
        }
    }

    fn resolve_names(&self) -> Result<Names, AppError> {
        let categories = self.category_repo.find_all()?;
        let tags = self.tag_repo.find_all()?;
        let projects = self.project_repo.find_all()?;
        let project_names = projects
            .into_iter()
            .map(|p| (p.id, p.name))
            .collect::<HashMap<_, _>>();
        Ok(Names {
            categories,
            tags,
            project_names,
        })
    }

    fn build_item(
        &self,
        p: &Prompt,
        names: &Names,
        render: bool,
        vars: &HashMap<String, String>,
    ) -> Result<ExportItem, AppError> {
        let rel = self.prompt_repo.get_relations(&p.id)?;
        let category_path = rel
            .category_id
            .as_deref()
            .map(|id| category_path(&names.categories, id))
            .unwrap_or_default();
        let tags = rel
            .tag_ids
            .iter()
            .map(|id| tag_display(&names.tags, id))
            .collect();
        let projects = rel
            .project_ids
            .iter()
            .filter_map(|id| names.project_names.get(id).cloned())
            .collect();

        let variables = self.extractor.extract_variables(&p.content);
        let content = if render {
            template::render(&p.content, &merge_defaults(&variables, vars))
        } else {
            p.content.clone()
        };

        Ok(ExportItem {
            id: p.id.clone(),
            title: p.title.clone(),
            description: p.description.clone(),
            content,
            rendered: render,
            category_path,
            tags,
            projects,
            variables,
            meta: ExportMeta {
                rating: p.rating,
                is_pinned: p.is_pinned,
                is_favorite: p.is_favorite,
                created_at: p.created_at.clone(),
                updated_at: p.updated_at.clone(),
            },
        })
    }
}

fn merge_defaults(
    definitions: &[VariableDefinition],
    overrides: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut m = HashMap::new();
    for v in definitions {
        if let Some(d) = &v.default_value {
            m.insert(v.name.clone(), d.clone());
        }
    }
    for (k, v) in overrides {
        m.insert(k.clone(), v.clone());
    }
    m
}

/// 分类路径：沿 `parent_id` 上溯到根，输出根 → 叶。
fn category_path(categories: &[Category], id: &str) -> Vec<String> {
    let map = categories
        .iter()
        .map(|c| (c.id.as_str(), c))
        .collect::<HashMap<_, _>>();
    let mut path = Vec::new();
    let mut cur = id.to_string();
    let mut guard = 0;
    while let Some(c) = map.get(cur.as_str()) {
        path.push(c.name.clone());
        match &c.parent_id {
            Some(parent) => cur = parent.clone(),
            None => break,
        }
        guard += 1;
        if guard > 64 {
            break;
        }
    }
    path.reverse();
    path
}

/// 标签显示名：沿 `parent_id` 上溯，输出「父/子」。
fn tag_display(tags: &[Tag], id: &str) -> String {
    let map = tags
        .iter()
        .map(|t| (t.id.as_str(), t))
        .collect::<HashMap<_, _>>();
    let mut chain = Vec::new();
    let mut cur = id.to_string();
    let mut guard = 0;
    while let Some(t) = map.get(cur.as_str()) {
        chain.push(t.name.clone());
        match &t.parent_id {
            Some(parent) => cur = parent.clone(),
            None => break,
        }
        guard += 1;
        if guard > 64 {
            break;
        }
    }
    chain.reverse();
    chain.join("/")
}

fn unique_slug(base: String, seen: &mut HashMap<String, usize>) -> String {
    let n = seen.entry(base.clone()).or_insert(0);
    *n += 1;
    if *n == 1 {
        base
    } else {
        format!("{base}-{n}")
    }
}

fn io_err(e: std::io::Error) -> AppError {
    AppError::Io(e.to_string())
}
