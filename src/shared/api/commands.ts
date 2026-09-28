import { invoke } from "@tauri-apps/api/core";

/**
 * IPC 契约 —— 前端单一事实源。
 * 命令名与 src-tauri/crates/ipc/src/commands/ 下的 #[tauri::command] 一一对应。
 * 变更契约时：先改本文件 + Rust 结构体，再分别实现两端（「契约先行」）。
 */

/** 连通性探针（M0 唯一开放命令） */
export async function ping(): Promise<string> {
  return invoke<string>("ping");
}

// ===== 待接入命令契约（按里程碑逐步开放）=====
// 提示词：get_prompts / get_prompt / create_prompt / update_prompt / delete_prompt
// 组织：  get_projects / get_tags ...
// 模板：  render_template / extract_variables
// 版本：  get_versions / diff_versions / rollback_version
// 同步：  sync_now / get_sync_status
// 安全：  unlock_vault / lock_vault / change_master_password