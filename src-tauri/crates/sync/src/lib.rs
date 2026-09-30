//! 同步引擎（对应架构方案 4.2、M4 分工清单第 6 章）。
//!
//! M4 起实现：
//! - [`webdav`]：WebDAV 客户端（`RemoteStore` 实现，Basic Auth）。
//! - [`crdt`]：HLC（混合逻辑时钟）+ 字段级 LWW 合并。
//! - [`engine`]：`SyncEngine` 实现，采集本地变更 + 加密上传 / 下载解密 + 冲突合并回写。
//!
//! 边界（M4 分工清单 §6）：本 crate 只依赖 `prompt_core` 的 trait（`SyncCipher` /
//! `SyncEngine` / `RemoteStore` 契约在 Phase 0 已冻结），不依赖 crypto / storage 的具体实现；
//! 本地变更日志通过 [`engine::SyncLogStore`] 抽象注入，测试用内存实现，生产由 IPC/存储层接线。

pub mod crdt;
pub mod engine;
pub mod webdav;

pub use crdt::{compare_entries, merge_entries, Hlc};
pub use engine::{Engine, SyncLogStore};
pub use webdav::WebDavClient;
