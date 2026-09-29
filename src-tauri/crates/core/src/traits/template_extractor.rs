use crate::models::VariableDefinition;

/// 模板变量提取抽象（M3 模板与版本，对应架构分工 §5 轨道 B）。
///
/// - 由 `crates/template` 提供实现（正则 `\{\{(\w+)\}\}`）。
/// - 命令层只面向此 trait 编程，不感知具体提取实现。
pub trait TemplateExtractor: Send + Sync {
    fn extract_variables(&self, content: &str) -> Vec<VariableDefinition>;
}
