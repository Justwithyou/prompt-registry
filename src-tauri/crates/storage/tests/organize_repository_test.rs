use prompt_core::{
    AppError, BatchUpdateInput, CategoryRepository, CreateCategoryInput, CreateProjectInput,
    CreatePromptInput, CreateTagInput, ProjectRepository, PromptQuery, PromptRelationsInput,
    PromptRepository, PromptSortBy, TagRepository, UpdateCategoryInput, UpdateProjectInput,
    UpdateTagInput,
};
use storage::{
    Db, SqliteCategoryRepository, SqliteProjectRepository, SqlitePromptRepository,
    SqliteTagRepository,
};

/// 共享同一内存库的四个仓库（模拟真实装配：同一条 SQLite 连接）。
fn shared_repos() -> (
    SqlitePromptRepository,
    SqliteProjectRepository,
    SqliteTagRepository,
    SqliteCategoryRepository,
) {
    let db = Db::open_in_memory().expect("打开内存数据库失败");
    (
        SqlitePromptRepository::new(db.clone()),
        SqliteProjectRepository::new(db.clone()),
        SqliteTagRepository::new(db.clone()),
        SqliteCategoryRepository::new(db),
    )
}

#[test]
fn project_crud_roundtrip() {
    let repo = SqliteProjectRepository::open_in_memory().unwrap();
    let created = repo
        .create(&CreateProjectInput {
            name: "项目A".into(),
            description: Some("描述".into()),
            color: Some("#ff0000".into()),
        })
        .unwrap();
    assert!(!created.id.is_empty());
    assert_eq!(created.name, "项目A");

    let all = repo.find_all().unwrap();
    assert_eq!(all.len(), 1);

    let updated = repo
        .update(
            &created.id,
            &UpdateProjectInput {
                name: Some("项目B".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(updated.name, "项目B");
    assert_eq!(updated.description.as_deref(), Some("描述"));

    repo.delete(&created.id).unwrap();
    assert!(repo.find_all().unwrap().is_empty());
}

#[test]
fn project_delete_not_found() {
    let repo = SqliteProjectRepository::open_in_memory().unwrap();
    let res = repo.delete("不存在");
    assert!(matches!(res, Err(AppError::NotFound(_))));
}

#[test]
fn tag_crud_with_hierarchy() {
    let repo = SqliteTagRepository::open_in_memory().unwrap();
    let parent = repo
        .create(&CreateTagInput {
            name: "场景".into(),
            ..Default::default()
        })
        .unwrap();
    let child = repo
        .create(&CreateTagInput {
            name: "客服".into(),
            parent_id: Some(parent.id.clone()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(child.parent_id.as_deref(), Some(parent.id.as_str()));

    let all = repo.find_all().unwrap();
    assert_eq!(all.len(), 2);

    let updated = repo
        .update(
            &child.id,
            &UpdateTagInput {
                color: Some("#00ff00".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(updated.color.as_deref(), Some("#00ff00"));

    repo.delete(&child.id).unwrap();
    assert_eq!(repo.find_all().unwrap().len(), 1);
}

#[test]
fn category_crud_with_sort_order() {
    let repo = SqliteCategoryRepository::open_in_memory().unwrap();
    let a = repo
        .create(&CreateCategoryInput {
            name: "分类A".into(),
            sort_order: Some(2),
            ..Default::default()
        })
        .unwrap();
    let b = repo
        .create(&CreateCategoryInput {
            name: "分类B".into(),
            sort_order: Some(1),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(a.sort_order, 2);
    assert_eq!(b.sort_order, 1);

    // 按 sort_order ASC 排序。
    let all = repo.find_all().unwrap();
    assert_eq!(all[0].name, "分类B");
    assert_eq!(all[1].name, "分类A");

    let updated = repo
        .update(
            &a.id,
            &UpdateCategoryInput {
                name: Some("分类A改名".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(updated.name, "分类A改名");

    repo.delete(&b.id).unwrap();
    assert_eq!(repo.find_all().unwrap().len(), 1);
}

#[test]
fn set_relations_assigns_project_tags_category() {
    let (prompt_repo, project_repo, tag_repo, category_repo) = shared_repos();

    let project = project_repo
        .create(&CreateProjectInput {
            name: "P".into(),
            ..Default::default()
        })
        .unwrap();
    let tag = tag_repo
        .create(&CreateTagInput {
            name: "T".into(),
            ..Default::default()
        })
        .unwrap();
    let category = category_repo
        .create(&CreateCategoryInput {
            name: "C".into(),
            ..Default::default()
        })
        .unwrap();

    let prompt = prompt_repo
        .create(&CreatePromptInput {
            title: "提示词".into(),
            content: "内容".into(),
            ..Default::default()
        })
        .unwrap();

    let updated = prompt_repo
        .set_relations(
            &prompt.id,
            &PromptRelationsInput {
                project_ids: Some(vec![project.id.clone()]),
                tag_ids: Some(vec![tag.id.clone()]),
                category_id: Some(category.id.clone()),
                is_favorite: Some(true),
                is_pinned: Some(true),
                rating: Some(4),
            },
        )
        .unwrap();

    assert_eq!(updated.category_id.as_deref(), Some(category.id.as_str()));
    assert!(updated.is_favorite);
    assert!(updated.is_pinned);
    assert_eq!(updated.rating, 4);

    // 关系落库可被查询命回。
    let by_project = prompt_repo
        .find_all(&PromptQuery {
            project_id: Some(project.id.clone()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(by_project.len(), 1);

    let by_tag = prompt_repo
        .find_all(&PromptQuery {
            tag_ids: Some(vec![tag.id.clone()]),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(by_tag.len(), 1);

    let by_category = prompt_repo
        .find_all(&PromptQuery {
            category_id: Some(category.id.clone()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(by_category.len(), 1);
}

#[test]
fn find_all_combined_filter_multi_tag_intersect() {
    let (prompt_repo, project_repo, tag_repo, category_repo) = shared_repos();

    let project = project_repo
        .create(&CreateProjectInput {
            name: "P".into(),
            ..Default::default()
        })
        .unwrap();
    let t1 = tag_repo
        .create(&CreateTagInput {
            name: "T1".into(),
            ..Default::default()
        })
        .unwrap();
    let t2 = tag_repo
        .create(&CreateTagInput {
            name: "T2".into(),
            ..Default::default()
        })
        .unwrap();
    let category = category_repo
        .create(&CreateCategoryInput {
            name: "C".into(),
            ..Default::default()
        })
        .unwrap();

    let p1 = prompt_repo
        .create(&CreatePromptInput {
            title: "甲".into(),
            content: "x".into(),
            ..Default::default()
        })
        .unwrap();
    let p2 = prompt_repo
        .create(&CreatePromptInput {
            title: "乙".into(),
            content: "y".into(),
            ..Default::default()
        })
        .unwrap();

    prompt_repo
        .set_relations(
            &p1.id,
            &PromptRelationsInput {
                project_ids: Some(vec![project.id.clone()]),
                tag_ids: Some(vec![t1.id.clone(), t2.id.clone()]),
                category_id: Some(category.id.clone()),
                is_favorite: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    prompt_repo
        .set_relations(
            &p2.id,
            &PromptRelationsInput {
                project_ids: Some(vec![project.id.clone()]),
                tag_ids: Some(vec![t1.id.clone()]),
                category_id: Some(category.id.clone()),
                is_favorite: Some(false),
                ..Default::default()
            },
        )
        .unwrap();

    // 项目筛选：两条都在该项目下。
    let by_project = prompt_repo
        .find_all(&PromptQuery {
            project_id: Some(project.id.clone()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(by_project.len(), 2);

    // 单标签：两条都有 T1。
    let by_t1 = prompt_repo
        .find_all(&PromptQuery {
            tag_ids: Some(vec![t1.id.clone()]),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(by_t1.len(), 2);

    // 多标签取交集：仅 P1 同时有 T1 + T2。
    let by_t1_t2 = prompt_repo
        .find_all(&PromptQuery {
            tag_ids: Some(vec![t1.id.clone(), t2.id.clone()]),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(by_t1_t2.len(), 1);
    assert_eq!(by_t1_t2[0].id, p1.id);

    // 分类筛选：两条同分类。
    let by_category = prompt_repo
        .find_all(&PromptQuery {
            category_id: Some(category.id.clone()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(by_category.len(), 2);

    // 收藏筛选。
    let by_favorite = prompt_repo
        .find_all(&PromptQuery {
            is_favorite: Some(true),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(by_favorite.len(), 1);
    assert_eq!(by_favorite[0].id, p1.id);

    // 联合：项目 + 多标签交集 + 收藏。
    let combined = prompt_repo
        .find_all(&PromptQuery {
            project_id: Some(project.id),
            tag_ids: Some(vec![t1.id, t2.id]),
            is_favorite: Some(true),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(combined.len(), 1);
    assert_eq!(combined[0].id, p1.id);
}

#[test]
fn find_all_sort_by_rating() {
    let repo = SqlitePromptRepository::open_in_memory().unwrap();
    let low = repo
        .create(&CreatePromptInput {
            title: "低分".into(),
            content: "x".into(),
            ..Default::default()
        })
        .unwrap();
    let high = repo
        .create(&CreatePromptInput {
            title: "高分".into(),
            content: "y".into(),
            ..Default::default()
        })
        .unwrap();

    repo.set_relations(
        &low.id,
        &PromptRelationsInput {
            rating: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    repo.set_relations(
        &high.id,
        &PromptRelationsInput {
            rating: Some(5),
            ..Default::default()
        },
    )
    .unwrap();

    let sorted = repo
        .find_all(&PromptQuery {
            sort_by: Some(PromptSortBy::Rating),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(sorted[0].id, high.id);
    assert_eq!(sorted[1].id, low.id);
}

#[test]
fn batch_update_add_tags_move_project_delete() {
    let (prompt_repo, project_repo, tag_repo, _) = shared_repos();

    let project = project_repo
        .create(&CreateProjectInput {
            name: "P".into(),
            ..Default::default()
        })
        .unwrap();
    let tag = tag_repo
        .create(&CreateTagInput {
            name: "T".into(),
            ..Default::default()
        })
        .unwrap();

    let p1 = prompt_repo
        .create(&CreatePromptInput {
            title: "一".into(),
            content: "x".into(),
            ..Default::default()
        })
        .unwrap();
    let p2 = prompt_repo
        .create(&CreatePromptInput {
            title: "二".into(),
            content: "y".into(),
            ..Default::default()
        })
        .unwrap();

    // 批量打标签 + 移动项目。
    prompt_repo
        .batch_update(
            &[p1.id.clone(), p2.id.clone()],
            &BatchUpdateInput {
                add_tag_ids: Some(vec![tag.id.clone()]),
                set_project_id: Some(project.id.clone()),
                ..Default::default()
            },
        )
        .unwrap();

    let by_tag = prompt_repo
        .find_all(&PromptQuery {
            tag_ids: Some(vec![tag.id.clone()]),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(by_tag.len(), 2);

    let by_project = prompt_repo
        .find_all(&PromptQuery {
            project_id: Some(project.id.clone()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(by_project.len(), 2);

    // 批量删除其中一条（软删除）。
    prompt_repo
        .batch_update(
            std::slice::from_ref(&p1.id),
            &BatchUpdateInput {
                delete: true,
                ..Default::default()
            },
        )
        .unwrap();

    assert!(prompt_repo.find_by_id(&p1.id).unwrap().is_none());
    assert!(prompt_repo.find_by_id(&p2.id).unwrap().is_some());

    let remaining = prompt_repo.find_all(&PromptQuery::default()).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, p2.id);
}

#[test]
fn set_relations_not_found() {
    let repo = SqlitePromptRepository::open_in_memory().unwrap();
    let res = repo.set_relations("不存在", &PromptRelationsInput::default());
    assert!(matches!(res, Err(AppError::NotFound(_))));
}

#[test]
fn batch_update_empty_ids_is_noop() {
    let repo = SqlitePromptRepository::open_in_memory().unwrap();
    repo.batch_update(&[], &BatchUpdateInput::default())
        .unwrap();
}
