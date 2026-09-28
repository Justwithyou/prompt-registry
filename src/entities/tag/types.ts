/**
 * 标签领域模型（前端类型，与 crates/core/src/models/tag.rs 对齐）。
 * 支持层级标签：parentId 指向父标签。
 */
export interface Tag {
  id: string;
  name: string;
  parentId: string | null;
  color: string | null;
}