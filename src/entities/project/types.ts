/**
 * 项目领域模型（前端类型，与 crates/core/src/models/project.rs 对齐）。
 */
export interface Project {
  id: string;
  name: string;
  description: string | null;
  createdAt: string;
  updatedAt: string;
}