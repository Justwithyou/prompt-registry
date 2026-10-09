use template::extract_variables;

#[test]
fn extracts_simple_variables() {
    let vars = extract_variables("你好 {{name}}，欢迎使用 {{tool}}");
    assert_eq!(vars.len(), 2);
    assert_eq!(vars[0].name, "name");
    assert_eq!(vars[1].name, "tool");
    // 其余字段本阶段为 None
    assert!(vars[0].default_value.is_none());
    assert!(vars[0].options.is_none());
    assert!(vars[0].description.is_none());
}

#[test]
fn deduplicates_repeated_variables() {
    let vars = extract_variables("{{a}} {{b}} {{a}}");
    assert_eq!(vars.len(), 2);
    assert_eq!(vars[0].name, "a");
    assert_eq!(vars[1].name, "b");
}

#[test]
fn extracts_underscores_and_digits() {
    let vars = extract_variables("{{user_name}} {{count2}}");
    assert_eq!(vars.len(), 2);
    assert_eq!(vars[0].name, "user_name");
    assert_eq!(vars[1].name, "count2");
}

#[test]
fn extracts_chinese_and_special_char_names() {
    let vars = extract_variables("{{主题}} {{城市}} {{user-name}} {{昵称1}}");
    assert_eq!(vars.len(), 4);
    assert_eq!(vars[0].name, "主题");
    assert_eq!(vars[1].name, "城市");
    assert_eq!(vars[2].name, "user-name");
    assert_eq!(vars[3].name, "昵称1");
}

#[test]
fn returns_empty_without_variables() {
    assert!(extract_variables("普通文本，无变量").is_empty());
    assert!(extract_variables("").is_empty());
}

#[test]
fn ignores_incomplete_braces() {
    let vars = extract_variables("{{name} 和 {other}} 与 {{ok}}");
    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0].name, "ok");
}
