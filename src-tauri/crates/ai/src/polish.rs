//! 「AI 调优」编排：对存量提示词正文做方向化改写。

use prompt_core::{AiConfig, AppError, PromptDraft};
use serde::Deserialize;

use crate::generate::parse_draft;
use crate::provider::Provider;

/// 调优选项：方向（更专业/更简洁/更具体）+ 补充指令。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolishOptions {
    pub style: Option<String>,
    pub instruction: Option<String>,
}

/// 调优一条提示词正文，返回调优后草稿（不落库）。
pub fn polish_prompt(
    provider: &dyn Provider,
    config: &AiConfig,
    api_key: &str,
    content: &str,
    options: &PolishOptions,
) -> Result<PromptDraft, AppError> {
    let system = build_polish_system(options);
    let raw = provider.generate(config, api_key, &system, content)?;
    let draft = parse_draft(&raw);
    let content = if draft.content.trim().is_empty() {
        raw.trim().to_string()
    } else {
        draft.content
    };
    let variables = template::extract_variables(&content);
    Ok(PromptDraft {
        title: String::new(),
        content,
        description: String::new(),
        suggested_tags: Vec::new(),
        suggested_category: None,
        variables,
    })
}

fn build_polish_system(opts: &PolishOptions) -> String {
    let mut s = String::from(
        "你是一名资深提示词工程专家。请对用户给出的提示词正文进行调优改写，\
         只输出改写后的正文，不要输出任何解释或前后缀。",
    );
    if let Some(style) = opts.style.as_deref().filter(|t| !t.trim().is_empty()) {
        s.push_str(&format!("调优方向：{}。", style.trim()));
    }
    if let Some(instr) = opts.instruction.as_deref().filter(|t| !t.trim().is_empty()) {
        s.push_str(&format!("额外要求：{}。", instr.trim()));
    }
    s.push_str(
        "原样保留正文中的 `{{变量}}` 占位，不展开、不删除、不改名。\
         只输出一个 JSON 对象：{\"content\":\"调优后的正文\"}，其余字段可空。",
    );
    s
}
