use std::collections::HashSet;
use std::sync::OnceLock;

use prompt_core::VariableDefinition;
use regex::Regex;

/// 返回编译好的模板变量正则（`\{\{(\w+)\}\}`），进程内仅编译一次。
fn variable_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\{\{(\w+)\}\}").expect("模板变量正则为常量，必然合法"))
}

/// 从模板正文中提取 `{{变量名}}` 列表（M3 分工清单 §3.2 冻结签名）。
///
/// 按首次出现顺序去重返回；仅返回变量名，其余字段
/// （`default_value` / `options` / `description`）本阶段置 `None`，由使用者后续补全。
pub fn extract_variables(content: &str) -> Vec<VariableDefinition> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for caps in variable_regex().captures_iter(content) {
        let Some(name) = caps.get(1) else {
            continue;
        };
        if seen.insert(name.as_str().to_string()) {
            out.push(VariableDefinition {
                name: name.as_str().to_string(),
                default_value: None,
                options: None,
                description: None,
            });
        }
    }

    out
}
