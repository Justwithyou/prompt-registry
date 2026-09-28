use serde::{Deserialize, Serialize};

/// 标签实体（支持层级：parent_id 指向父标签）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub color: Option<String>,
}