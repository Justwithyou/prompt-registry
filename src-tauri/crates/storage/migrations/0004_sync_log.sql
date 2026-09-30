-- 0004_sync_log.sql
-- M4 同步层迁移：本地变更日志 + WebDAV 配置 + 设备标识。

-- 本地同步变更日志（collect → 加密上传 → 合并回写 → 清除已同步）。
CREATE TABLE IF NOT EXISTS sync_log (
  id          TEXT PRIMARY KEY,
  entity_type TEXT NOT NULL,
  entity_id   TEXT NOT NULL,
  operation   TEXT NOT NULL,
  payload     TEXT NOT NULL,
  hlc         TEXT NOT NULL,
  device_id   TEXT NOT NULL,
  created_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sync_log_entity ON sync_log(entity_type, entity_id);

-- WebDAV 同步配置（单行，id 恒为 1）。
CREATE TABLE IF NOT EXISTS sync_config (
  id       INTEGER PRIMARY KEY CHECK (id = 1),
  url      TEXT NOT NULL,
  username TEXT NOT NULL,
  token    TEXT,
  enabled  INTEGER NOT NULL DEFAULT 0
);

-- 设备标识等杂项元数据（键值对，如 device_id）。
CREATE TABLE IF NOT EXISTS sync_meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);