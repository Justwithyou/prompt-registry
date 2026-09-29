pub mod prompt;

/// 连通性探针命令（M0）。
///
/// 用于验证「前端 invoke → Tauri → Rust 命令 → 回传」最小链路是否打通。
/// 后续业务命令（get_prompts / create_prompt ...）按领域拆分到 `commands/prompt.rs` 等文件。
#[tauri::command]
pub fn ping() -> String {
    "pong".to_string()
}
