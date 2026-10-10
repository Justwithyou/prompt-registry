//! 脱敏：出网前把敏感内容替换为占位符，回网后再原文还原。
//!
//! 仅作用于 `user` 侧文本与模型输出；映射只存内存，不落盘、不进日志、不随请求发出。

use std::collections::HashMap;

use regex::Regex;

/// 脱敏结果：脱敏后的文本 + 编号 → 原文 的映射（仅内存）。
pub struct MaskedText {
    /// 可安全出网的脱敏文本。
    pub text: String,
    map: HashMap<String, String>,
}

impl MaskedText {
    /// 回网后还原：把输出中的占位符替换回原文。
    pub fn restore(&self, output: &str) -> String {
        let mut out = output.to_string();
        for (id, original) in &self.map {
            out = out.replace(id, original);
        }
        out
    }
}

/// 判断 baseUrl 是否指向本地端点（本地模型默认不脱敏、不要求 Key）。
pub fn is_local(base_url: &str) -> bool {
    let u = base_url.trim().to_lowercase();
    u.contains("://localhost")
        || u.contains("://127.0.0.1")
        || u.contains("://::1")
        || u.contains("://[::1]")
}

/// 对文本脱敏：用户自定义敏感词（精确）+ 内置正则心态叠加，命中项编号、同名复用。
pub fn sanitize(text: &str, custom_terms: &[String]) -> MaskedText {
    let mut map: HashMap<String, String> = HashMap::new();
    let mut text = text.to_string();

    // 先精确替换自定义词（公司名/人名/项目代号等）。
    for term in custom_terms
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        if text.contains(term) {
            let id = mask(&mut map, term);
            text = text.replace(term, &id);
        }
    }

    // 再跑内置正则规则（手机号/邮箱/身份证/银行卡/IPv4）。
    for re in builtin_patterns() {
        text = re
            .replace_all(&text, |caps: &regex::Captures| mask(&mut map, &caps[0]))
            .into_owned();
    }

    MaskedText { text, map }
}

/// 内置敏感规则。
fn builtin_patterns() -> Vec<Regex> {
    vec![
        // 手机号
        Regex::new(r"\b1[3-9]\d{9}\b").unwrap(),
        // 邮箱
        Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").unwrap(),
        // 身份证（18 位）
        Regex::new(r"\b\d{17}[\dXx]\b").unwrap(),
        // 银行卡（16~19 位）
        Regex::new(r"\b\d{16,19}\b").unwrap(),
        // IPv4
        Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap(),
    ]
}

/// 为原文分配占位符编号；若原文已出现过则复用同一编号。
fn mask(map: &mut HashMap<String, String>, original: &str) -> String {
    if let Some((id, _)) = map.iter().find(|(_, v)| v.as_str() == original) {
        return id.clone();
    }
    let id = format!("[[SENSITIVE_{:02}]]", map.len() + 1);
    map.insert(id.clone(), original.to_string());
    id
}
