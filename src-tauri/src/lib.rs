use std::sync::Arc;

use prompt_core::{
    CategoryRepository, ProjectRepository, PromptRepository, TagRepository, TemplateExtractor,
    VersionRepository,
};
use tauri::Manager;

use ipc::AppState;
use storage::{
    Db, SqliteCategoryRepository, SqliteProjectRepository, SqlitePromptRepository,
    SqliteTagRepository, SqliteVersionRepository,
};
use template::RegexTemplateExtractor;

/// 应用初始化入口（对应架构方案 4.3）。
///
/// M2 装配：在应用数据目录打开（或创建）明文 SQLite 数据库 `prompt.db`，
/// 四个组织层仓库（提示词 / 项目 / 标签 / 分类）共享同一条 `Db` 连接；
/// M3 起扩展版本仓库（同一 `Db`）与模板变量提取器。
/// 构造后统一注入 `AppState`，注册 ping + 提示词七命令 + 组织层十四命令 + 模板版本四命令。
/// 加密 / 密钥派生归 M4，本阶段不引入 SQLCipher。
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("prompt.db");

            let db = Db::open(&db_path)?;
            let prompt_repo: Arc<dyn PromptRepository> =
                Arc::new(SqlitePromptRepository::new(db.clone()));
            let project_repo: Arc<dyn ProjectRepository> =
                Arc::new(SqliteProjectRepository::new(db.clone()));
            let tag_repo: Arc<dyn TagRepository> = Arc::new(SqliteTagRepository::new(db.clone()));
            let category_repo: Arc<dyn CategoryRepository> =
                Arc::new(SqliteCategoryRepository::new(db.clone()));
            let version_repo: Arc<dyn VersionRepository> =
                Arc::new(SqliteVersionRepository::new(db));
            let template_extractor: Arc<dyn TemplateExtractor> = Arc::new(RegexTemplateExtractor);

            app.manage(AppState {
                prompt_repo,
                project_repo,
                tag_repo,
                category_repo,
                version_repo,
                template_extractor,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::ping,
            ipc::commands::prompt::get_prompts,
            ipc::commands::prompt::create_prompt,
            ipc::commands::prompt::update_prompt,
            ipc::commands::prompt::delete_prompt,
            ipc::commands::prompt::search_prompts,
            ipc::commands::prompt::set_prompt_relations,
            ipc::commands::prompt::batch_update_prompts,
            ipc::commands::category::list_categories,
            ipc::commands::category::create_category,
            ipc::commands::category::update_category,
            ipc::commands::category::delete_category,
            ipc::commands::project::list_projects,
            ipc::commands::project::create_project,
            ipc::commands::project::update_project,
            ipc::commands::project::delete_project,
            ipc::commands::tag::list_tags,
            ipc::commands::tag::create_tag,
            ipc::commands::tag::update_tag,
            ipc::commands::tag::delete_tag,
            ipc::commands::template::extract_variables,
            ipc::commands::version::list_prompt_versions,
            ipc::commands::version::get_prompt_version,
            ipc::commands::version::rollback_prompt_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};

    /// 构造一个注入了共享内存数据库六个能力、并注册提示词 + 模板 + 版本命令的 mock 应用。
    fn mock_app() -> tauri::App<tauri::test::MockRuntime> {
        let db = Db::open_in_memory().expect("打开内存数据库失败");
        let prompt_repo: Arc<dyn PromptRepository> =
            Arc::new(SqlitePromptRepository::new(db.clone()));
        let project_repo: Arc<dyn ProjectRepository> =
            Arc::new(SqliteProjectRepository::new(db.clone()));
        let tag_repo: Arc<dyn TagRepository> = Arc::new(SqliteTagRepository::new(db.clone()));
        let category_repo: Arc<dyn CategoryRepository> =
            Arc::new(SqliteCategoryRepository::new(db.clone()));
        let version_repo: Arc<dyn VersionRepository> = Arc::new(SqliteVersionRepository::new(db));
        let template_extractor: Arc<dyn TemplateExtractor> = Arc::new(RegexTemplateExtractor);

        mock_builder()
            .manage(AppState {
                prompt_repo,
                project_repo,
                tag_repo,
                category_repo,
                version_repo,
                template_extractor,
            })
            .invoke_handler(tauri::generate_handler![
                ipc::commands::prompt::get_prompts,
                ipc::commands::prompt::create_prompt,
                ipc::commands::prompt::update_prompt,
                ipc::commands::prompt::delete_prompt,
                ipc::commands::prompt::search_prompts,
                ipc::commands::template::extract_variables,
                ipc::commands::version::list_prompt_versions,
            ])
            .build(mock_context(noop_assets()))
            .expect("failed to build mock app")
    }

    /// M1-B3 冒烟：装配后 `get_prompts` 空库返回 `[]`。
    #[test]
    fn get_prompts_empty_db_returns_empty() {
        let app = mock_app();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("failed to build webview");

        let res = get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "get_prompts".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(serde_json::json!({ "query": {} })),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .expect("invoke get_prompts 失败");

        let prompts: Vec<prompt_core::Prompt> = res.deserialize().expect("反序列化结果失败");
        assert!(prompts.is_empty(), "空库 get_prompts 应返回空列表");
    }

    /// M1-B3 冒烟：创建后 `get_prompts` 能取回一条记录，验证 create → 列表 端到端。
    #[test]
    fn create_then_get_prompts_roundtrip() {
        let app = mock_app();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("failed to build webview");

        let created = get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "create_prompt".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(serde_json::json!({
                    "input": { "title": "冒烟", "content": "内容" }
                })),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .expect("invoke create_prompt 失败");
        let created: prompt_core::Prompt = created.deserialize().expect("反序列化创建结果失败");
        assert_eq!(created.title, "冒烟");

        let listed = get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "get_prompts".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(serde_json::json!({ "query": {} })),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .expect("invoke get_prompts 失败");
        let prompts: Vec<prompt_core::Prompt> = listed.deserialize().expect("反序列化列表失败");
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].id, created.id);
    }

    /// M3-B3 冒烟：`extract_variables` 能从模板正文中提取 `{{变量}}` 列表。
    #[test]
    fn extract_variables_returns_definitions() {
        let app = mock_app();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("failed to build webview");

        let res = get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "extract_variables".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(serde_json::json!({
                    "content": "你好 {{name}}，欢迎来到 {{city}}。"
                })),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .expect("invoke extract_variables 失败");

        let vars: Vec<prompt_core::VariableDefinition> =
            res.deserialize().expect("反序列化变量列表失败");
        assert_eq!(vars.len(), 2, "应提取到两个变量");
        assert_eq!(vars[0].name, "name");
        assert_eq!(vars[1].name, "city");
    }

    /// M3-B3 冒烟：`list_prompt_versions` 空库返回 `[]`。
    #[test]
    fn list_prompt_versions_empty_db_returns_empty() {
        let app = mock_app();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("failed to build webview");

        let res = get_ipc_response(
            &webview,
            tauri::webview::InvokeRequest {
                cmd: "list_prompt_versions".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(serde_json::json!({
                    "promptId": "nonexistent"
                })),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .expect("invoke list_prompt_versions 失败");

        let versions: Vec<prompt_core::PromptVersion> =
            res.deserialize().expect("反序列化版本列表失败");
        assert!(
            versions.is_empty(),
            "空库 list_prompt_versions 应返回空列表"
        );
    }
}
