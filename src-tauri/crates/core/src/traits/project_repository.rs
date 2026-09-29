use crate::errors::AppError;
use crate::models::{CreateProjectInput, Project, UpdateProjectInput};

/// 项目仓库抽象（M2 组织层）。
///
/// - 由 `storage` crate 提供 SQLite 实现（轨道 A）。
/// - 命令层只面向此 trait 编程，不感知具体存储引擎。
pub trait ProjectRepository: Send + Sync {
    fn find_all(&self) -> Result<Vec<Project>, AppError>;
    fn create(&self, input: &CreateProjectInput) -> Result<Project, AppError>;
    fn update(&self, id: &str, input: &UpdateProjectInput) -> Result<Project, AppError>;
    fn delete(&self, id: &str) -> Result<(), AppError>;
}
