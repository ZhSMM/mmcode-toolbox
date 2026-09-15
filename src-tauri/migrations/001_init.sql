-- 001_init.sql
-- 初始 schema。Tauri 启动时执行。

CREATE TABLE IF NOT EXISTS tools (
    tool_id     TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    category    TEXT NOT NULL,
    route       TEXT NOT NULL,
    icon        TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    enabled     INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_tools_category ON tools(category);

CREATE TABLE IF NOT EXISTS history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    tool_id     TEXT NOT NULL,
    input       TEXT,
    output      TEXT,
    status      TEXT NOT NULL DEFAULT 'success',
    error_msg   TEXT,
    created_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_history_tool ON history(tool_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_history_time ON history(created_at DESC);

CREATE TABLE IF NOT EXISTS favorites (
    tool_id     TEXT PRIMARY KEY,
    created_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS app_settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  INTEGER NOT NULL
);