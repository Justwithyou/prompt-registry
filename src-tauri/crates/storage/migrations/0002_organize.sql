-- 0002_organize.sql
-- M2 组织层迁移：分类（树形）+ 项目 + 多对多关联 + 层级标签。

-- 分类（树形，支持拖拽排序）
CREATE TABLE IF NOT EXISTS categories (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  parent_id TEXT REFERENCES categories(id),
  sort_order INTEGER DEFAULT 0,
  created_at TEXT NOT NULL
);

-- 方案 A：prompts 增加 category_id 外键（树形分类稳定 ID）。
-- 旧 prompts.category 字符串列保留兼容读，本列供「按分类筛选」使用。
ALTER TABLE prompts ADD COLUMN category_id TEXT REFERENCES categories(id);

-- 项目
CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  color TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- 提示词-项目 多对多
CREATE TABLE IF NOT EXISTS prompt_projects (
  prompt_id TEXT NOT NULL REFERENCES prompts(id),
  project_id TEXT NOT NULL REFERENCES projects(id),
  PRIMARY KEY (prompt_id, project_id)
);

-- 标签（层级：场景/客服）
CREATE TABLE IF NOT EXISTS tags (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  parent_id TEXT REFERENCES tags(id),
  color TEXT,
  created_at TEXT NOT NULL
);

-- 提示词-标签 多对多
CREATE TABLE IF NOT EXISTS prompt_tags (
  prompt_id TEXT NOT NULL REFERENCES prompts(id),
  tag_id TEXT NOT NULL REFERENCES tags(id),
  PRIMARY KEY (prompt_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_prompts_category_id ON prompts(category_id);
CREATE INDEX IF NOT EXISTS idx_tags_parent ON tags(parent_id);
CREATE INDEX IF NOT EXISTS idx_prompt_projects_project ON prompt_projects(project_id);
CREATE INDEX IF NOT EXISTS idx_prompt_tags_tag ON prompt_tags(tag_id);