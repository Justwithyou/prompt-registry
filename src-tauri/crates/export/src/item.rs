//! 导出中间表示（IR）与格式 / 选择器类型。

use prompt_core::VariableDefinition;
use serde::{Deserialize, Serialize};

/// 导出格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Skill,
    Markdown,
    Json,
}

impl ExportFormat {
    /// 从命令行 / UI 传入的字符串解析格式，非法时返回 `None`。
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "skill" | "sk" => Some(Self::Skill),
            "markdown" | "md" => Some(Self::Markdown),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

/// 导出选择器，对应设计文档的三类粒度（单条 / 分类 / 标签 / 项目）。
#[derive(Debug, Clone)]
pub enum ExportSelector {
    /// 全部（CLI `list` 不带维度时）。
    All,
    Ids(Vec<String>),
    Categories(Vec<String>),
    Tags(Vec<String>),
    Projects(Vec<String>),
}

/// IPC 入参形式的选择器（前端传 camelCase JSON，缺省字段为空）。
///
/// 优先级：`ids` > `categoryIds` > `projectIds` > `tagIds` > 全部。
/// 分类 / 标签 / 项目均支持多选（并集导出），对应 M7 批量导出优化。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ExportSelectorInput {
    pub ids: Vec<String>,
    pub category_ids: Vec<String>,
    pub tag_ids: Vec<String>,
    pub project_ids: Vec<String>,
}

impl ExportSelectorInput {
    pub fn to_selector(&self) -> ExportSelector {
        if !self.ids.is_empty() {
            ExportSelector::Ids(self.ids.clone())
        } else if !self.category_ids.is_empty() {
            ExportSelector::Categories(self.category_ids.clone())
        } else if !self.project_ids.is_empty() {
            ExportSelector::Projects(self.project_ids.clone())
        } else if !self.tag_ids.is_empty() {
            ExportSelector::Tags(self.tag_ids.clone())
        } else {
            ExportSelector::All
        }
    }
}

/// 附带元数据。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportMeta {
    pub rating: i32,
    pub is_pinned: bool,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 导出中间表示：与存储字段解耦，序列化层只消费此结构。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    /// 正文（`rendered` 为真时是填值后的成品，否则是原文）。
    pub content: String,
    pub rendered: bool,
    /// 分类路径（根 → 叶，如 `["工作", "周报"]`）。
    pub category_path: Vec<String>,
    /// 标签显示名（已展开层级，如 `"场景/客服"`）。
    pub tags: Vec<String>,
    /// 项目名列表。
    pub projects: Vec<String>,
    pub variables: Vec<VariableDefinition>,
    pub meta: ExportMeta,
}
