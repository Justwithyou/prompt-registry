//! prompt-cli 入口：装配仓库与导出内核，分发子命令。

mod args;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use clap::Parser;
use export::{ExportFormat, ExportItem, ExportSelector, ExportService};
use prompt_core::{
    CategoryRepository, ProjectRepository, PromptRepository, TagRepository, TemplateExtractor,
};
use storage::{
    Db, SqliteCategoryRepository, SqliteProjectRepository, SqlitePromptRepository,
    SqliteTagRepository,
};
use template::RegexTemplateExtractor;

use args::{parse_vars, Cli, Commands};

fn main() {
    if let Err(e) = run() {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    let db = Db::open(cli.db.as_path()).map_err(|e| e.to_string())?;
    let prompt_repo: Arc<dyn PromptRepository> = Arc::new(SqlitePromptRepository::new(db.clone()));
    let category_repo: Arc<dyn CategoryRepository> =
        Arc::new(SqliteCategoryRepository::new(db.clone()));
    let tag_repo: Arc<dyn TagRepository> = Arc::new(SqliteTagRepository::new(db.clone()));
    let project_repo: Arc<dyn ProjectRepository> =
        Arc::new(SqliteProjectRepository::new(db.clone()));
    let extractor: Arc<dyn TemplateExtractor> = Arc::new(RegexTemplateExtractor);
    let svc = ExportService::new(
        prompt_repo,
        category_repo,
        tag_repo,
        project_repo,
        extractor,
    );

    match cli.command {
        Commands::List {
            category,
            tag,
            project,
            keyword,
            format,
        } => {
            let selector = selector_from(category, tag, project);
            let items = svc
                .items(&selector, keyword.as_deref(), false, &HashMap::new())
                .map_err(|e| e.to_string())?;
            if format == "json" {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&items).map_err(|e| e.to_string())?
                );
            } else {
                print_table(&items);
            }
        }
        Commands::Get { id, format } => {
            let fmt = ExportFormat::parse(&format).expect("clap 已校验取值");
            let items = svc
                .items(&ExportSelector::Ids(vec![id]), None, false, &HashMap::new())
                .map_err(|e| e.to_string())?;
            let item = items.into_iter().next().ok_or("未找到该提示词")?;
            println!("{}", export::writer::serialize(&item, fmt).trim_end());
        }
        Commands::Export {
            category,
            tag,
            project,
            format,
            out,
            render,
            var,
        } => {
            let selector = selector_from(category, tag, project);
            let fmt = ExportFormat::parse(&format).expect("clap 已校验取值");
            let vars = parse_vars(&var)?;
            let written = svc
                .export(&selector, fmt, render, &vars, Path::new(&out))
                .map_err(|e| e.to_string())?;
            for w in written {
                println!("{w}");
            }
        }
        Commands::Run { id, var, out } => {
            let vars = parse_vars(&var)?;
            let rendered = svc.render_one(&id, &vars).map_err(|e| e.to_string())?;
            match out {
                Some(path) => std::fs::write(&path, rendered).map_err(|e| e.to_string())?,
                None => println!("{}", rendered.trim_end()),
            }
        }
    }
    Ok(())
}

fn selector_from(
    category: Option<String>,
    tag: Vec<String>,
    project: Option<String>,
) -> ExportSelector {
    if let Some(c) = category {
        ExportSelector::Categories(vec![c])
    } else if let Some(p) = project {
        ExportSelector::Projects(vec![p])
    } else if !tag.is_empty() {
        ExportSelector::Tags(tag)
    } else {
        ExportSelector::All
    }
}

fn print_table(items: &[ExportItem]) {
    if items.is_empty() {
        println!("(无提示词)");
        return;
    }
    println!("ID | 标题 | 分类 | 标签 | 项目");
    println!("--- | --- | --- | --- | ---");
    for it in items {
        println!(
            "{} | {} | {} | {} | {}",
            short_id(&it.id),
            it.title,
            it.category_path.join("/"),
            it.tags.join(","),
            it.projects.join(","),
        );
    }
}

fn short_id(id: &str) -> &str {
    if id.len() > 8 {
        &id[..8]
    } else {
        id
    }
}
