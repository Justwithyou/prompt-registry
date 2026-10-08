//! 序列化 writer：把 [`ExportItem`] 渲染为 SKILL.md / Markdown / JSON 文本。

use prompt_core::VariableDefinition;

use crate::item::{ExportFormat, ExportItem};

/// 标题 → 文件名 / 目录名的 slug（保留中英文数字，空格与符号折叠为 `-`，限制长度）。
pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for c in title.trim().chars() {
        if c.is_alphanumeric() {
            out.push(c);
            last_dash = false;
        } else if !out.is_empty() && !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        return "untitled".to_string();
    }
    let mut chars = trimmed.chars();
    let head: String = chars.by_ref().take(64).collect();
    if trimmed.chars().count() > 64 {
        head
    } else {
        trimmed.to_string()
    }
}

/// 序列化单条 IR 为指定格式的完整文本。
pub fn serialize(item: &ExportItem, format: ExportFormat) -> String {
    match format {
        ExportFormat::Skill => skill_md(item),
        ExportFormat::Markdown => markdown(item),
        ExportFormat::Json => serde_json::to_string_pretty(item).unwrap_or_else(|_| "{}".to_string()),
    }
}

/// 变量说明表（含变量时才返回 Some）。
fn variables_table(vars: &[VariableDefinition]) -> Option<String> {
    if vars.is_empty() {
        return None;
    }
    let mut t = String::from("## 变量\n\n| 变量 | 默认值 | 可选值 | 说明 |\n| --- | --- | --- | --- |\n");
    for v in vars {
        let default = v.default_value.as_deref().unwrap_or("-").replace("\n", " ");
        let options = match &v.options {
            Some(o) if !o.is_empty() => o.join(", "),
            _ => "-".to_string(),
        };
        let desc = v.description.as_deref().unwrap_or("-").replace("\n", " ");
        t.push_str(&format!("| {} | {} | {} | {} |\n", v.name, default, options, desc));
    }
    Some(t)
}

fn markdown(item: &ExportItem) -> String {
    let mut s = String::new();
    s.push_str(&format!("# {}\n", item.title));
    if let Some(desc) = &item.description {
        if !desc.trim().is_empty() {
            s.push_str(&format!("\n{}\n", desc));
        }
    }
    s.push_str(&format!("\n{}\n", item.content));
    if let Some(table) = variables_table(&item.variables) {
        s.push_str(&format!("\n{}\n", table));
    }
    s
}

fn skill_md(item: &ExportItem) -> String {
    let mut s = String::from("---\n");
    s.push_str(&format!("name: {}\n", yaml_quote(&item.title)));
    let desc = item
        .description
        .clone()
        .filter(|d| !d.trim().is_empty())
        .unwrap_or_else(|| format!("提示词「{}」", item.title));
    s.push_str(&format!("description: {}\n", yaml_quote(&desc)));

    if !item.category_path.is_empty() {
        s.push_str(&format!("category: {}\n", yaml_quote(&item.category_path.join("/"))));
    }
    if !item.tags.is_empty() {
        s.push_str(&format!("tags: [{}]\n", yaml_list(&item.tags)));
    }
    if !item.projects.is_empty() {
        s.push_str(&format!("projects: [{}]\n", yaml_list(&item.projects)));
    }

    s.push_str("metadata:\n");
    s.push_str(&format!("  rating: {}\n", item.meta.rating));
    s.push_str(&format!("  pinned: {}\n", item.meta.is_pinned));
    s.push_str(&format!("  favorite: {}\n", item.meta.is_favorite));
    s.push_str(&format!("  source: {}\n", yaml_quote(&item.id)));
    s.push_str("---\n\n");

    s.push_str(&format!("# {}\n\n", item.title));
    s.push_str(&format!("{}\n", item.content));
    if let Some(table) = variables_table(&item.variables) {
        s.push_str(&format!("\n{}\n", table));
    }
    s
}

/// YAML 标量：统一双引号包裹并转义，避免冒号 / 换行 / 引号破坏 frontmatter。
fn yaml_quote(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// YAML 内联列表元素（逐项双引号包裹）。
fn yaml_list(items: &[String]) -> String {
    items
        .iter()
        .map(|s| yaml_quote(s))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use prompt_core::VariableDefinition;

    fn item() -> ExportItem {
        ExportItem {
            id: "abc".to_string(),
            title: "周报模板".to_string(),
            description: Some("生成周报".to_string()),
            content: "你好 {{city}}".to_string(),
            rendered: false,
            category_path: vec!["工作".to_string(), "周报".to_string()],
            tags: vec!["模板".to_string()],
            projects: vec!["内部流程".to_string()],
            variables: vec![VariableDefinition {
                name: "city".to_string(),
                default_value: Some("上海".to_string()),
                options: None,
                description: None,
            }],
            meta: crate::item::ExportMeta {
                rating: 5,
                is_pinned: true,
                is_favorite: false,
                created_at: "t0".to_string(),
                updated_at: "t1".to_string(),
            },
        }
    }

    #[test]
    fn slugify_keeps_cjk_and_dashes() {
        assert_eq!(slugify("周报模板"), "周报模板");
        assert_eq!(slugify("客服-退款话术"), "客服-退款话术");
        assert_eq!(slugify("  a  b  "), "a-b");
    }

    #[test]
    fn json_is_valid() {
        let v: serde_json::Value = serde_json::from_str(&serialize(&item(), ExportFormat::Json)).unwrap();
        assert_eq!(v["title"], "周报模板");
    }

    #[test]
    fn skill_has_frontmatter() {
        let s = serialize(&item(), ExportFormat::Skill);
        assert!(s.starts_with("---\nname: \"周报模板\"\n"));
        assert!(s.contains("category: \"工作/周报\""));
        assert!(s.contains("tags: [\"模板\"]"));
        assert!(s.contains("metadata:"));
    }
}