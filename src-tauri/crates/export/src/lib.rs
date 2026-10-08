//! export crate：提示词资产的单一导出内核（对应设计文档 §3.1）。
//!
//! - `item`：导出中间表示（IR）、格式与选择器类型；
//! - `writer`：SKILL.md / Markdown / JSON 三种序列化；
//! - `service`：读数据（repositories）→ 拼装 IR →（可选）渲染 → 写文件。
//!
//! `cli` 与 `ipc` 均复用 `ExportService`，保证两处输出口径一致。

pub mod item;
pub mod service;
pub mod writer;

pub use item::{ExportFormat, ExportItem, ExportMeta, ExportSelector, ExportSelectorInput};
pub use service::ExportService;