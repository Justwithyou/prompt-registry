//! template crate：模板变量提取（M3 模板与版本）。
//!
//! 契约（M3-P1）：`extract_variables(content) -> Vec<VariableDefinition>`，
//! `VariableDefinition` 单一事实源定义在 `prompt_core`，本 crate 复用该类型。
//! 并提供一个零开销的 [`RegexTemplateExtractor`] 实现 core 的 `TemplateExtractor` trait，
//! 供装配阶段以 `Arc<dyn TemplateExtractor>` 形式注入 AppState。

use prompt_core::{TemplateExtractor, VariableDefinition};

pub mod extractor;
pub mod render;

pub use extractor::extract_variables;
pub use render::render;

/// 基于正则的模板变量提取器（满足 core `TemplateExtractor` trait）。
///
/// 零大小类型，无内部状态，可安全共享于多线程上下文（`Send + Sync`）。
#[derive(Debug, Default)]
pub struct RegexTemplateExtractor;

impl TemplateExtractor for RegexTemplateExtractor {
    fn extract_variables(&self, content: &str) -> Vec<VariableDefinition> {
        extractor::extract_variables(content)
    }
}
