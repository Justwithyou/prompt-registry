-- 0001_init.sql
-- M1 初始迁移：提示词主表 + FTS5 全文索引 + 常用索引。
-- 说明：M1 使用明文 SQLite；加密 / 密钥派生归 M4。

CREATE TABLE IF NOT EXISTS prompts (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    content     TEXT NOT NULL,
    description TEXT,
    category    TEXT,
    is_pinned   INTEGER NOT NULL DEFAULT 0,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    rating      INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    deleted_at  TEXT
);

-- 分类 / 置顶 / 更新时间 常用索引（列表默认按置顶 + 更新时间倒序）。
CREATE INDEX IF NOT EXISTS idx_prompts_category ON prompts (category);
CREATE INDEX IF NOT EXISTS idx_prompts_is_pinned ON prompts (is_pinned);
CREATE INDEX IF NOT EXISTS idx_prompts_updated_at ON prompts (updated_at DESC);

-- FTS5 全文搜索虚表。trigram 分词器对中英文子串匹配均可用。
CREATE VIRTUAL TABLE IF NOT EXISTS prompts_fts USING fts5 (
    id UNINDEXED,
    title,
    content,
    description,
    tokenize = 'trigram'
);