use serde::{Deserialize, Serialize};

/// 模板变量定义（`extract_variables` 产物，对应 M3 分工清单 §3.2）。
///
/// 序列化契约：`rename_all = "camelCase"`，前端 `VariableDefinition` 与之一一对应。
/// 单一事实源定义在 core；`crates/template` 的提取器复用本类型返回。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableDefinition {
    pub name: String,
    pub default_value: Option<String>,
    pub options: Option<Vec<String>>,
    pub description: Option<String>,
}
