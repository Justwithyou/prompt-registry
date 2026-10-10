//! 「AI 提取变量」编排：把写死的文案语义化改写为带 `{{变量}}` 占位的正文。

use std::collections::HashMap;

use prompt_core::{AiConfig, AppError, PromptDraft, VariableDefinition};

use crate::generate::parse_draft;
use crate::provider::Provider;

/// 用模型把正文参数化改写，返回改写后正文 + 变量定义（不落库）。
pub fn extract_variables_ai(
    provider: &dyn Provider,
    config: &AiConfig,
    api_key: &str,
    content: &str,
) -> Result<PromptDraft, AppError> {
    let system = build_extract_system();
    let raw = provider.generate(config, api_key, &system, content)?;
    let draft = parse_draft(&raw);
    let rewritten = if draft.content.trim().is_empty() {
        raw.trim().to_string()
    } else {
        draft.content
    };
    let content_vars = template::extract_variables(&rewritten);
    let variables = merge_variables(&draft.variables, &content_vars);
    Ok(PromptDraft {
        title: String::new(),
        content: rewritten,
        description: String::new(),
        suggested_tags: Vec::new(),
        suggested_category: None,
        variables,
    })
}

/// 合并模型自报变量定义与正文实际占位：`name` 以正文为准，元数据尽量沿用模型。
fn merge_variables(
    from_model: &[VariableDefinition],
    from_content: &[VariableDefinition],
) -> Vec<VariableDefinition> {
    let mut defs: HashMap<String, VariableDefinition> = HashMap::new();
    for v in from_model {
        if !v.name.trim().is_empty() {
            defs.insert(v.name.clone(), v.clone());
        }
    }
    from_content
        .iter()
        .map(|e| defs.get(&e.name).cloned().unwrap_or_else(|| e.clone()))
        .collect()
}

fn build_extract_system() -> String {
    "你是一名资深提示词工程专家。请分析用户给出的提示词正文，\
     找出其中会随场景变化的部分（对象/称呼/数值/部门/日期等），改写为带 `{{变量名}}` 占位的版本；\
     变量名用中文描述含义（如 `{{收件人}}`、`{{部门}}`）。\
     只输出一个 JSON 对象：{\"content\":\"改写后的正文\"}。\
     不要输出任何解释或前后缀；不改变原意、不新增无关内容。"
        .to_string()
}
