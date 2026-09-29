use serde::{Deserialize, Serialize};

/// 分类实体（树形，支持拖拽排序）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
}

/// 创建分类入参。`name` 必填。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryInput {
    pub name: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

/// 更新分类入参（字段级局部更新，None 表示不修改）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateCategoryInput {
    pub name: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: Option<i32>,
}
