//! IPC 层：Tauri Commands 注册与参数校验（对应架构方案 4.2 的 ipc crate）。
//! 命令函数集中在本 crate 的 `commands` 模块，由 src-tauri/src/lib.rs 统一注册。

pub mod commands;