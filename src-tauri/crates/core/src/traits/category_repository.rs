use crate::errors::AppError;
use crate::models::{Category, CreateCategoryInput, UpdateCategoryInput};

/// 分类仓库抽象（M2 组织层，树形分类，支持拖拽排序）。
///
/// - 由 `storage` crate 提供 SQLite 实现（轨道 A）。
/// - 命令层只面向此 trait 编程，不感知具体存储引擎。
pub trait CategoryRepository: Send + Sync {
    fn find_all(&self) -> Result<Vec<Category>, AppError>;
    fn create(&self, input: &CreateCategoryInput) -> Result<Category, AppError>;
    fn update(&self, id: &str, input: &UpdateCategoryInput) -> Result<Category, AppError>;
    fn delete(&self, id: &str) -> Result<(), AppError>;
}
