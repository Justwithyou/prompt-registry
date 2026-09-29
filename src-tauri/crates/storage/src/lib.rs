//! 数据访问层（对应架构方案 4.4）。
//!
//! M1 起以明文 SQLite（rusqlite `bundled`）实现 `PromptRepository`（`SqlitePromptRepository`），
//! 并管理数据库迁移。加密 / 密钥派生归 M4，本 crate 当前不引入 SQLCipher。

pub mod repository;
pub mod sqlite;

pub use repository::SqlitePromptRepository;
pub use sqlite::Connection;
