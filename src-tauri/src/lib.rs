use std::sync::Arc;

use prompt_core::PromptRepository;
use tauri::Manager;

use ipc::AppState;
use storage::SqlitePromptRepository;

/// 应用初始化入口（对应架构方案 4.3）。
///
/// M1 装配：在应用数据目录打开（或创建）明文 SQLite 数据库 `prompt.db`，
/// 构造 `SqlitePromptRepository` 注入 `AppState`，并注册 ping + 提示词五命令。
/// 加密 / 密钥派生归 M4，本阶段不引入 SQLCipher。
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("prompt.db");

            let repo: Arc<dyn PromptRepository> = Arc::new(SqlitePromptRepository::open(&db_path)?);
            app.manage(AppState { prompt_repo: repo });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::ping,
            ipc::commands::prompt::get_prompts,
            ipc::commands::prompt::create_prompt,
            ipc::commands::prompt::update_prompt,
            ipc::commands::prompt::delete_prompt,
            ipc::commands::prompt::search_prompts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};

    /// 构造一个注入了内存数据库 Repository、只注册提示词命令的 mock 应用。
    fn mock_app() -> tauri::App<tauri::test::MockRuntime> {
        let repo: Arc<dyn PromptRepository> =
            Arc::new(SqlitePromptRepository::open_in_memory().expect("打开内存数据库失败"));

        mock_builder()
            .manage(AppState { prompt_repo: repo })
            .invoke_handler(tauri::generate_handler![
                ipc::commands::prompt::get_prompts,
                ipc::commands::prompt::create_prompt,
                ipc::commands::prompt::update_prompt,
                ipc::commands::prompt::delete_prompt,
                ipc::commands::prompt::search_prompts,
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
}
