use prompt_core::{
    AppError, CreatePromptInput, PromptRepository, UpdatePromptInput, VersionRepository,
};
use storage::{Db, SqlitePromptRepository, SqliteVersionRepository};

/// 共享同一内存库的提示词仓库与版本仓库（模拟真实装配：同一条 SQLite 连接）。
fn shared() -> (SqlitePromptRepository, SqliteVersionRepository) {
    let db = Db::open_in_memory().expect("打开内存数据库失败");
    (
        SqlitePromptRepository::new(db.clone()),
        SqliteVersionRepository::new(db),
    )
}

fn create_prompt(repo: &SqlitePromptRepository, content: &str) -> String {
    repo.create(&CreatePromptInput {
        title: "标题".into(),
        content: content.into(),
        ..Default::default()
    })
    .unwrap()
    .id
}

#[test]
fn list_empty_before_any_update() {
    let (prompt_repo, version_repo) = shared();
    let id = create_prompt(&prompt_repo, "内容v0");
    assert!(version_repo.list(&id).unwrap().is_empty());
}

#[test]
fn update_records_version_with_increment() {
    let (prompt_repo, version_repo) = shared();
    let id = create_prompt(&prompt_repo, "内容v0");

    prompt_repo
        .update(
            &id,
            &UpdatePromptInput {
                content: Some("内容v1".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let versions = version_repo.list(&id).unwrap();
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].version, "0.1.0");
    assert_eq!(versions[0].content, "内容v0");
    assert_eq!(versions[0].change_summary.as_deref(), Some("更新内容"));

    prompt_repo
        .update(
            &id,
            &UpdatePromptInput {
                content: Some("内容v2".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let versions = version_repo.list(&id).unwrap();
    assert_eq!(versions.len(), 2);
    // 最新在前。
    assert_eq!(versions[0].version, "0.1.1");
    assert_eq!(versions[0].content, "内容v1");
    assert_eq!(versions[1].version, "0.1.0");
}

#[test]
fn title_update_uses_title_summary() {
    let (prompt_repo, version_repo) = shared();
    let id = create_prompt(&prompt_repo, "内容v0");

    prompt_repo
        .update(
            &id,
            &UpdatePromptInput {
                title: Some("新标题".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let versions = version_repo.list(&id).unwrap();
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].change_summary.as_deref(), Some("更新标题"));
    // 仅改标题也会归档旧内容。
    assert_eq!(versions[0].content, "内容v0");
}

#[test]
fn get_version_and_not_found() {
    let (prompt_repo, version_repo) = shared();
    let id = create_prompt(&prompt_repo, "内容v0");
    prompt_repo
        .update(
            &id,
            &UpdatePromptInput {
                content: Some("内容v1".into()),
                ..Default::default()
            },
        )
        .unwrap();

    let versions = version_repo.list(&id).unwrap();
    let got = version_repo.get(&versions[0].id).unwrap();
    assert_eq!(got.id, versions[0].id);
    assert_eq!(got.content, "内容v0");

    let err = version_repo.get("不存在").unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
}

#[test]
fn rollback_restores_content_and_appends_version() {
    let (prompt_repo, version_repo) = shared();
    let id = create_prompt(&prompt_repo, "内容v0");
    prompt_repo
        .update(
            &id,
            &UpdatePromptInput {
                content: Some("内容v1".into()),
                ..Default::default()
            },
        )
        .unwrap();
    prompt_repo
        .update(
            &id,
            &UpdatePromptInput {
                content: Some("内容v2".into()),
                ..Default::default()
            },
        )
        .unwrap();

    let versions = version_repo.list(&id).unwrap();
    assert_eq!(versions.len(), 2);
    // 回滚到最老的 0.1.0（内容v0）。
    let target_id = versions[1].id.clone();

    let rolled = version_repo.rollback(&id, &target_id).unwrap();
    assert_eq!(rolled.content, "内容v0");

    // 回滚追加一条新版本（不覆盖历史）。
    let versions = version_repo.list(&id).unwrap();
    assert_eq!(versions.len(), 3);
    assert_eq!(versions[0].version, "0.1.2");
    // 新版本记录回滚前的内容（与 update 的旧内容归档语义一致）。
    assert_eq!(versions[0].content, "内容v2");
    assert_eq!(versions[0].change_summary.as_deref(), Some("回滚到 0.1.0"));

    // 历史未被删除。
    assert_eq!(versions[2].version, "0.1.0");
    assert_eq!(versions[2].content, "内容v0");
}

#[test]
fn rollback_rejects_foreign_version() {
    let (prompt_repo, version_repo) = shared();
    let id_a = create_prompt(&prompt_repo, "A");
    let id_b = create_prompt(&prompt_repo, "B");
    prompt_repo
        .update(
            &id_a,
            &UpdatePromptInput {
                content: Some("A2".into()),
                ..Default::default()
            },
        )
        .unwrap();

    let version_of_a = version_repo.list(&id_a).unwrap()[0].id.clone();
    let err = version_repo.rollback(&id_b, &version_of_a).unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
}
