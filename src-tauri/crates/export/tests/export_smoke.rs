//! export 内核端到端冒烟：seed → IR → 渲染 → 导出文件。

use std::collections::HashMap;
use std::sync::Arc;

use export::{ExportFormat, ExportSelector, ExportService};
use prompt_core::{
    CategoryRepository, CreateCategoryInput, CreatePromptInput, CreateTagInput,
    ProjectRepository, PromptRelationsInput, PromptRepository, TagRepository, TemplateExtractor,
};
use storage::{
    Db, SqliteCategoryRepository, SqliteProjectRepository, SqlitePromptRepository,
    SqliteTagRepository,
};
use template::RegexTemplateExtractor;

fn service(db: &Db) -> ExportService {
    ExportService::new(
        Arc::new(SqlitePromptRepository::new(db.clone())),
        Arc::new(SqliteCategoryRepository::new(db.clone())),
        Arc::new(SqliteTagRepository::new(db.clone())),
        Arc::new(SqliteProjectRepository::new(db.clone())),
        Arc::new(RegexTemplateExtractor) as Arc<dyn TemplateExtractor>,
    )
}

#[test]
fn assemble_render_and_export_skill() {
    let db = Db::open_in_memory().expect("open mem db");

    let work = db_category(&db, "工作", None);
    let weekly = db_category(&db, "周报", Some(work.id.clone()));
    let tag = db_tag(&db, "模板");
    let project = db_project(&db, "内部流程");

    let prompt_repo = SqlitePromptRepository::new(db.clone());
    let created = prompt_repo
        .create(&CreatePromptInput {
            title: "周报模板".into(),
            content: "你好 {{city}}，请生成周报。".into(),
            description: Some("生成周报".into()),
            category: None,
        })
        .expect("create prompt");

    prompt_repo
        .set_relations(
            &created.id,
            &PromptRelationsInput {
                category_id: Some(weekly.id.clone()),
                tag_ids: Some(vec![tag.id.clone()]),
                project_ids: Some(vec![project.id.clone()]),
                ..Default::default()
            },
        )
        .expect("set relations");

    let svc = service(&db);

    // IR 拼装与层级展开
    let items = svc
        .items(&ExportSelector::Ids(vec![created.id.clone()]), None, false, &HashMap::new())
        .expect("items");
    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item.category_path, vec!["工作", "周报"]);
    assert_eq!(item.tags, vec!["模板"]);
    assert_eq!(item.projects, vec!["内部流程"]);
    assert_eq!(item.variables.len(), 1);
    assert_eq!(item.variables[0].name, "city");

    // 渲染成品
    let mut vars = HashMap::new();
    vars.insert("city".to_string(), "上海".to_string());
    let rendered = svc.render_one(&created.id, &vars).expect("render");
    assert_eq!(rendered, "你好 上海，请生成周报。");

    // 导出 SKILL.md
    let out = std::env::temp_dir().join(format!("prompt-export-smoke-{}", created.id));
    let written = svc
        .export(&ExportSelector::Ids(vec![created.id.clone()]), ExportFormat::Skill, false, &HashMap::new(), &out)
        .expect("export");
    assert_eq!(written.len(), 1);
    let text = std::fs::read_to_string(&written[0]).expect("read skill");
    assert!(text.starts_with("---\nname: \"周报模板\"\n"));
    assert!(text.contains("category: \"工作/周报\""));
    assert!(text.contains("# 周报模板"));
    let _ = std::fs::remove_dir_all(&out);
}

fn db_category(db: &Db, name: &str, parent: Option<String>) -> prompt_core::Category {
    Arc::new(SqliteCategoryRepository::new(db.clone()))
        .create(&CreateCategoryInput {
            name: name.into(),
            parent_id: parent,
            sort_order: None,
        })
        .unwrap()
}

fn db_tag(db: &Db, name: &str) -> prompt_core::Tag {
    Arc::new(SqliteTagRepository::new(db.clone()))
        .create(&CreateTagInput {
            name: name.into(),
            parent_id: None,
            color: None,
        })
        .unwrap()
}

fn db_project(db: &Db, name: &str) -> prompt_core::Project {
    Arc::new(SqliteProjectRepository::new(db.clone()))
        .create(&prompt_core::CreateProjectInput {
            name: name.into(),
            description: None,
            color: None,
        })
        .unwrap()
}