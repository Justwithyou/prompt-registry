use std::collections::HashMap;

use crate::extractor::variable_regex;

/// 渲染模板：将正文中的 `{{name}}` 回填为 `vars` 中同名值。
///
/// 仅替换 `vars` 提供的变量；未覆盖的变量保留原 `{{name}}` 占位，交由下游决定是否补全。
/// 值合并（显式输入 > `default_value`）由调用方完成，本函数保持单一职责。
pub fn render(content: &str, vars: &HashMap<String, String>) -> String {
    variable_regex()
        .replace_all(content, |caps: &regex::Captures<'_>| {
            let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            match vars.get(name) {
                Some(value) => value.clone(),
                None => format!("{{{{{name}}}}}"),
            }
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_replaces_known_and_keeps_unknown() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "张三".to_string());
        let out = render("你好 {{name}}，欢迎来到 {{city}}。", &vars);
        assert_eq!(out, "你好 张三，欢迎来到 {{city}}。");
    }

    #[test]
    fn render_without_vars_keeps_placeholders() {
        let out = render("{{a}} {{b}}", &HashMap::new());
        assert_eq!(out, "{{a}} {{b}}");
    }
}
