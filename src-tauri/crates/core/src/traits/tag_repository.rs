use crate::errors::AppError;
use crate::models::{CreateTagInput, Tag, UpdateTagInput};

/// 标签仓库抽象（M2 组织层，层级标签）。
///
/// - 由 `storage` crate 提供 SQLite 实现（轨道 A）。
/// - 命令层只面向此 trait 编程，不感知具体存储引擎。
pub trait TagRepository: Send + Sync {
    fn find_all(&self) -> Result<Vec<Tag>, AppError>;
    fn create(&self, input: &CreateTagInput) -> Result<Tag, AppError>;
    fn update(&self, id: &str, input: &UpdateTagInput) -> Result<Tag, AppError>;
    fn delete(&self, id: &str) -> Result<(), AppError>;
}
