//! `generate_prompt` 编排逻辑的单元测试（注入 mock Provider，不触真实网络）。
//!
//! 覆盖三条路径：JSON 解析成功、纯文本兜底、变量提取与模板契约一致。

use ai::{generate_prompt, GenerateOptions, Provider};
use prompt_core::{AiConfig, AppError};

/// mock Provider：返回预设文本。
struct MockProvider(String);

impl Provider for MockProvider {
    fn generate(
        &self,
        _cfg: &AiConfig,
        _api_key: &str,
        _system: &str,
        _user: &str,
    ) -> Result<String, AppError> {
        Ok(self.0.clone())
    }
}

fn config() -> AiConfig {
    AiConfig::default()
}

#[test]
fn parses_structured_json_and_extracts_variables() {
    let raw = r#"{"title":"周报生成器","content":"请根据 {{主题}} 生成一份每周周报，侧重 {{重点}}。","description":"周报模板","suggestedTags":["周报","办公"],"suggestedCategory":"效率"}"#;
    let provider = MockProvider(raw.to_string());
    let draft = generate_prompt(
        &provider,
        &config(),
        "sk",
        "帮我写周报",
        &GenerateOptions::default(),
    )
    .expect("生成失败");

    assert_eq!(draft.title, "周报生成器");
    assert_eq!(draft.suggested_category.as_deref(), Some("效率"));
    assert_eq!(draft.suggested_tags, vec!["周报", "办公"]);
    assert_eq!(draft.variables.len(), 2);
    assert_eq!(draft.variables[0].name, "主题");
    assert_eq!(draft.variables[1].name, "重点");
}

#[test]
fn wraps_json_with_markdown_fence() {
    let raw = "```json\n{\"title\":\"T\",\"content\":\"C\",\"description\":\"\",\"suggestedTags\":[],\"suggestedCategory\":\"\"}\n```";
    let provider = MockProvider(raw.to_string());
    let draft = generate_prompt(
        &provider,
        &config(),
        "sk",
        "需求",
        &GenerateOptions::default(),
    )
    .expect("生成失败");
    assert_eq!(draft.title, "T");
    assert_eq!(draft.content, "C");
}

#[test]
fn falls_back_to_raw_text_as_content() {
    let raw = "这是模型返回的一段纯文本，没有 JSON。";
    let provider = MockProvider(raw.to_string());
    let draft = generate_prompt(
        &provider,
        &config(),
        "sk",
        "需求",
        &GenerateOptions::default(),
    )
    .expect("生成失败");
    assert_eq!(draft.content, raw);
    assert_eq!(draft.title, "");
    assert!(draft.variables.is_empty());
}
