use prompt_core::{AppError, CreatePromptInput, PromptQuery, PromptRepository, UpdatePromptInput};
use storage::SqlitePromptRepository;

fn repo() -> SqlitePromptRepository {
    SqlitePromptRepository::open_in_memory().expect("打开内存数据库失败")
}

#[test]
fn create_and_find_by_id() {
    let repo = repo();
    let created = repo
        .create(&CreatePromptInput {
            title: "测试提示词".into(),
            content: "这是内容".into(),
            description: Some("描述".into()),
            category: Some("分类A".into()),
        })
        .unwrap();

    assert!(!created.id.is_empty());
    assert_eq!(created.title, "测试提示词");
    assert_eq!(created.content, "这是内容");
    assert_eq!(created.description.as_deref(), Some("描述"));
    assert_eq!(created.category.as_deref(), Some("分类A"));
    assert!(!created.is_pinned);
    assert!(!created.is_favorite);
    assert_eq!(created.rating, 0);
    assert_eq!(created.created_at, created.updated_at);
    assert!(created.deleted_at.is_none());

    let found = repo.find_by_id(&created.id).unwrap().expect("应能找到");
    assert_eq!(found.id, created.id);
    assert_eq!(found.title, "测试提示词");
}

#[test]
fn find_all_with_keyword_and_limit() {
    let repo = repo();
    repo.create(&CreatePromptInput {
        title: "苹果".into(),
        content: "红苹果".into(),
        ..Default::default()
    })
    .unwrap();
    repo.create(&CreatePromptInput {
        title: "香蕉".into(),
        content: "黄香蕉".into(),
        ..Default::default()
    })
    .unwrap();

    let all = repo.find_all(&PromptQuery::default()).unwrap();
    assert_eq!(all.len(), 2);

    let filtered = repo
        .find_all(&PromptQuery {
            keyword: Some("苹果".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].title, "苹果");

    let limited = repo
        .find_all(&PromptQuery {
            limit: Some(1),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(limited.len(), 1);
}

#[test]
fn update_fields() {
    let repo = repo();
    let created = repo
        .create(&CreatePromptInput {
            title: "旧".into(),
            content: "旧内容".into(),
            ..Default::default()
        })
        .unwrap();

    let updated = repo
        .update(
            &created.id,
            &UpdatePromptInput {
                title: Some("新".into()),
                is_pinned: Some(true),
                rating: Some(5),
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(updated.title, "新");
    assert_eq!(updated.content, "旧内容");
    assert!(updated.is_pinned);
    assert_eq!(updated.rating, 5);
}

#[test]
fn update_not_found() {
    let repo = repo();
    let res = repo.update("不存在", &UpdatePromptInput::default());
    assert!(matches!(res, Err(AppError::NotFound(_))));
}

#[test]
fn delete_is_soft() {
    let repo = repo();
    let created = repo
        .create(&CreatePromptInput {
            title: "删除我".into(),
            content: "x".into(),
            ..Default::default()
        })
        .unwrap();

    repo.delete(&created.id).unwrap();

    assert!(repo.find_by_id(&created.id).unwrap().is_none());
    assert!(repo.find_all(&PromptQuery::default()).unwrap().is_empty());
}

#[test]
fn delete_not_found() {
    let repo = repo();
    let res = repo.delete("不存在");
    assert!(matches!(res, Err(AppError::NotFound(_))));
}

#[test]
fn search_fts_long_keyword() {
    let repo = repo();
    repo.create(&CreatePromptInput {
        title: "代码审查提示词".into(),
        content: "用于代码审查的完整模板".into(),
        ..Default::default()
    })
    .unwrap();
    repo.create(&CreatePromptInput {
        title: "写作提示词".into(),
        content: "用于写作的模板".into(),
        ..Default::default()
    })
    .unwrap();

    let hits = repo.search("代码审查").unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title, "代码审查提示词");
}

#[test]
fn search_short_keyword_falls_back_to_like() {
    let repo = repo();
    repo.create(&CreatePromptInput {
        title: "代码审查提示词".into(),
        content: "用于代码审查的完整模板".into(),
        ..Default::default()
    })
    .unwrap();
    repo.create(&CreatePromptInput {
        title: "写作提示词".into(),
        content: "用于写作的模板".into(),
        ..Default::default()
    })
    .unwrap();

    let hits = repo.search("写作").unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title, "写作提示词");
}
