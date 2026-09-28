/// 应用初始化入口（对应架构方案 4.3）。
///
/// M0 骨架：仅注册连通性命令 `ping`，用于验证「前端 invoke → Rust 命令 → 回传」链路。
/// M1 起在此接入：
///   - AppState（db / crypto / sync_engine）经 `app.manage(...)` 注入
///   - 各业务命令（ipc::commands::prompt::* 等）
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ipc::commands::ping])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}