//! SQLCipher 数据访问层（对应架构方案 4.4）。
//!
//! M1 起实现 `PromptRepository`（`SqlitePromptRepository`），并管理数据库迁移。
//! 依赖：tauri-plugin-rusqlite2 + rusqlite(features=["bundled-sqlcipher"])。
//! 后续需添加：prompt_core = { package = "core", path = "../core" }。