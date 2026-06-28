pub const CREATE_COMMAND_HISTORY_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS command_history (
  id TEXT PRIMARY KEY,
  action TEXT NOT NULL,
  project_path TEXT NOT NULL,
  commands_json TEXT NOT NULL,
  success INTEGER NOT NULL,
  exit_code INTEGER,
  stdout TEXT,
  stderr TEXT,
  started_at TEXT NOT NULL,
  finished_at TEXT NOT NULL,
  duration_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_command_history_started_at
ON command_history(started_at DESC);

CREATE INDEX IF NOT EXISTS idx_command_history_project_path
ON command_history(project_path);
"#;
