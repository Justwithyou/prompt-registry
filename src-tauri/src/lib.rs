use std::path::Path;
use std::sync::Arc;

use prompt_core::{
    AppError, CategoryRepository, ProjectRepository, PromptRepository, SyncCipher, SyncConfigStore,
    SyncEngine, SyncLogStore, TagRepository, TemplateExtractor, VaultEnvelope, VaultProvider,
    VersionRepository,
};
use tauri::Manager;

use crypto::{AesGcmCipher, CryptoVault, KeychainStore};
use ipc::AppState;
use storage::{
    Db, SqliteCategoryRepository, SqliteProjectRepository, SqlitePromptRepository,
    SqliteSyncLogStore, SqliteTagRepository, SqliteVersionRepository,
};
use sync::Engine;
use template::RegexTemplateExtractor;

mod ai_commands;
mod ai_config;
mod data_transfer;
mod storage_location;

/// 同步能力三元组（变更日志 / 配置仓库 / 引擎），供 `assemble_sync` 返回值复用。
type SyncAssembly = (
    Arc<dyn SyncLogStore>,
    Arc<dyn SyncConfigStore>,
    Arc<dyn SyncEngine>,
);

/// 装配同步能力：共享同一 `Db` 的变更日志 / 配置仓库 + 真正的同步引擎。
///
/// 返回三元组分别注入 `AppState` 的 `sync_log_store` / `sync_config_store` / `sync_engine`。
fn assemble_sync(db: &Db) -> Result<SyncAssembly, prompt_core::AppError> {
    let store = Arc::new(SqliteSyncLogStore::new(db.clone())?);
    let device_id = store.device_id().to_string();
    let log_store: Arc<dyn SyncLogStore> = store.clone();
    let config_store: Arc<dyn SyncConfigStore> = store.clone();
    let cipher: Arc<dyn SyncCipher> = Arc::new(AesGcmCipher);
    let engine: Arc<dyn SyncEngine> = Arc::new(Engine::new(cipher, log_store.clone(), device_id));
    Ok((log_store, config_store, engine))
}

/// 读取持久化的解锁信封（`vault.json`）；文件不存在时返回 `None`。
fn read_envelope(path: &Path) -> Result<Option<VaultEnvelope>, AppError> {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            let envelope =
                serde_json::from_str(&text).map_err(|e| AppError::Crypto(e.to_string()))?;
            Ok(Some(envelope))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(AppError::Io(e.to_string())),
    }
}

/// 应用初始化入口（对应架构方案 4.3）。
///
/// M2 装配：在应用数据目录打开（或创建）明文 SQLite 数据库 `prompt.db`，
/// 四个组织层仓库（提示词 / 项目 / 标签 / 分类）共享同一条 `Db` 连接；
/// M3 起扩展版本仓库（同一 `Db`）与模板变量提取器。
/// 构造后统一注入 `AppState`，注册 ping + 提示词七命令 + 组织层十四命令 + 模板版本四命令。
/// 加密 / 密钥派生归 M4，本阶段不引入 SQLCipher。
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let base_dir = app.path().app_data_dir()?;
            let data_dir = storage_location::load_dir(&base_dir);
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("prompt.db");
            let vault_file = data_dir.join("vault.json");

            // M4 装配收尾（遗留 #2/#3）：注入 OS 钥匙串、恢复持久化解锁信封并尝试自动解锁，
            // 再按「是否已设置口令 / 是否已解锁」三态打开数据库。
            let vault_concrete = CryptoVault::with_secret_store(Box::new(KeychainStore::new()?));
            if let Some(envelope) = read_envelope(&vault_file)? {
                vault_concrete.restore_envelope(envelope)?;
            }
            // 钥匙串中若存有 Master Key（用户开启过自动解锁），无需口令即可载入；
            // 无钥匙串 / 无条目 / 长度异常时不阻断启动，交由下方三态判定。
            let _ = vault_concrete.auto_unlock();

            let db = if vault_concrete.export_envelope().is_none() {
                // 从未设置口令：明文打开（setup_vault 时再就地加密）。
                Db::open(&db_path)?
            } else {
                match vault_concrete.master_key() {
                    // 已解锁（含自动解锁）：以 Master Key 打开加密库。
                    Ok(key) => Db::open_with_key(&db_path, Some(&key))?,
                    // 已设置口令但未解锁：先挂内存占位库，unlock_vault 时重开真实加密库。
                    Err(_) => Db::open_in_memory()?,
                }
            };
            // 让存储位置命令通过 State<Db> 取到同一共享连接，实现运行时热切换。
            app.manage(db.clone());
            let prompt_repo: Arc<dyn PromptRepository> =
                Arc::new(SqlitePromptRepository::new(db.clone()));
            let project_repo: Arc<dyn ProjectRepository> =
                Arc::new(SqliteProjectRepository::new(db.clone()));
            let tag_repo: Arc<dyn TagRepository> = Arc::new(SqliteTagRepository::new(db.clone()));
            let category_repo: Arc<dyn CategoryRepository> =
                Arc::new(SqliteCategoryRepository::new(db.clone()));
            let version_repo: Arc<dyn VersionRepository> =
                Arc::new(SqliteVersionRepository::new(db.clone()));
            let template_extractor: Arc<dyn TemplateExtractor> = Arc::new(RegexTemplateExtractor);
            // 同步变更日志 + 配置仓库 + 引擎完整装配（M4-C 闭环）。
            let vault: Arc<dyn VaultProvider> = Arc::new(vault_concrete);
            let (sync_log_store, sync_config_store, sync_engine) = assemble_sync(&db)?;

            app.manage(AppState {
                prompt_repo,
                project_repo,
                tag_repo,
                category_repo,
                version_repo,
                template_extractor,
                vault,
                sync_log_store,
                sync_config_store,
                sync_engine,
                db_path,
                vault_file,
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
            ipc::commands::prompt::get_prompt_relations,
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
            ipc::commands::vault::setup_vault,
            ipc::commands::vault::unlock_vault,
            ipc::commands::vault::lock_vault,
            ipc::commands::vault::change_password,
            ipc::commands::vault::is_locked,
            ipc::commands::vault::set_auto_unlock,
            ipc::commands::sync::configure_sync,
            ipc::commands::sync::sync_now,
            ipc::commands::sync::get_sync_status,
            ipc::commands::export::render_prompt,
            ipc::commands::export::export_prompts,
            ipc::commands::export::export_skill,
            storage_location::get_storage_location,
            storage_location::set_storage_location,
            data_transfer::export_data,
            data_transfer::import_data,
            data_transfer::import_db,
            ai_config::get_ai_config,
            ai_config::set_ai_config,
            ai_commands::set_ai_api_key,
            ai_commands::ai_generate_prompt,
            ai_commands::ai_polish_prompt,
            ai_commands::ai_extract_variables,
        ])
        .run(tauri::generate_context!())
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};

    /// 为每次 `mock_app()` 生成唯一临时目录的递增序号，避免多测试串扰。
    static MOCK_SEQ: AtomicU64 = AtomicU64::new(0);

    /// 构造一个注入了共享数据库六个能力、并注册提示词 + 模板 + 版本 + 保险库命令的 mock 应用。
    /// 数据库落临时目录文件（而非内存），使 `setup_vault` / `unlock_vault` 的 rekey / reopen 真实可用。
    fn mock_app() -> tauri::App<tauri::test::MockRuntime> {
        let dir = std::env::temp_dir().join(format!(
            "prompt-manager-mock-{}-{}",
            std::process::id(),
            MOCK_SEQ.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&dir).expect("创建 mock 临时目录失败");
        let db_path = dir.join("prompt.db");
        let vault_file = dir.join("vault.json");

        let db = Db::open(&db_path).expect("打开 mock 数据库失败");
        let prompt_repo: Arc<dyn PromptRepository> =
            Arc::new(SqlitePromptRepository::new(db.clone()));
        let project_repo: Arc<dyn ProjectRepository> =
            Arc::new(SqliteProjectRepository::new(db.clone()));
        let tag_repo: Arc<dyn TagRepository> = Arc::new(SqliteTagRepository::new(db.clone()));
        let category_repo: Arc<dyn CategoryRepository> =
            Arc::new(SqliteCategoryRepository::new(db.clone()));
        let version_repo: Arc<dyn VersionRepository> =
            Arc::new(SqliteVersionRepository::new(db.clone()));
        let template_extractor: Arc<dyn TemplateExtractor> = Arc::new(RegexTemplateExtractor);
        let vault: Arc<dyn VaultProvider> = Arc::new(CryptoVault::new());
        let (sync_log_store, sync_config_store, sync_engine) =
            assemble_sync(&db).expect("装配同步能力失败");

        mock_builder()
            .manage(db.clone())
            .manage(AppState {
                prompt_repo,
                project_repo,
                tag_repo,
                category_repo,
                version_repo,
                template_extractor,
                vault,
                sync_log_store,
                sync_config_store,
                sync_engine,
                db_path,
                vault_file,
            })
            .invoke_handler(tauri::generate_handler![
                ipc::commands::prompt::get_prompts,
                ipc::commands::prompt::create_prompt,
                ipc::commands::prompt::update_prompt,
                ipc::commands::prompt::delete_prompt,
                ipc::commands::prompt::search_prompts,
                ipc::commands::template::extract_variables,
                ipc::commands::version::list_prompt_versions,
                ipc::commands::vault::setup_vault,
                ipc::commands::vault::unlock_vault,
                ipc::commands::vault::lock_vault,
                ipc::commands::vault::is_locked,
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

    /// 对 mock webview 发起一次命令调用并取回可反序列化的响应体。
    fn invoke(
        webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        cmd: &str,
        body: serde_json::Value,
    ) -> tauri::ipc::InvokeResponseBody {
        get_ipc_response(
            webview,
            tauri::webview::InvokeRequest {
                cmd: cmd.into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .expect("invoke 失败")
    }

    /// M4-D3 冒烟：`is_locked` 初始（未 setup）应为锁定态。
    #[test]
    fn is_locked_initial_state() {
        let app = mock_app();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("failed to build webview");

        let res = invoke(&webview, "is_locked", serde_json::json!({}));
        let locked: bool = res.deserialize().expect("反序列化 is_locked 失败");
        assert!(locked, "初始（未 setup）is_locked 应为 true");
    }

    /// M4-D3 冒烟：setup → 未锁定 → lock → 锁定 → unlock → 未锁定 全链路。
    #[test]
    fn setup_lock_unlock_roundtrip() {
        let app = mock_app();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("failed to build webview");

        invoke(
            &webview,
            "setup_vault",
            serde_json::json!({ "password": "pw-123" }),
        );

        let unlocked = invoke(&webview, "is_locked", serde_json::json!({}));
        let locked: bool = unlocked.deserialize().expect("反序列化 is_locked 失败");
        assert!(!locked, "setup 后 is_locked 应为 false");

        invoke(&webview, "lock_vault", serde_json::json!({}));

        let relocked = invoke(&webview, "is_locked", serde_json::json!({}));
        let locked: bool = relocked.deserialize().expect("反序列化 is_locked 失败");
        assert!(locked, "lock 后 is_locked 应为 true");

        invoke(
            &webview,
            "unlock_vault",
            serde_json::json!({ "password": "pw-123" }),
        );

        let reunlocked = invoke(&webview, "is_locked", serde_json::json!({}));
        let locked: bool = reunlocked.deserialize().expect("反序列化 is_locked 失败");
        assert!(!locked, "unlock 后 is_locked 应为 false");
    }
}
