use crate::core::app_error::{AppError, AppResult};
use crate::core::command_runner::DisplayCommand;
use crate::storage::sqlite::CREATE_COMMAND_HISTORY_SQL;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandExecutionResult {
    pub id: String,
    pub action: String,
    pub project_path: String,
    pub commands: Vec<DisplayCommand>,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u128,
}

pub type CommandHistoryRecord = CommandExecutionResult;

pub struct HistoryStore {
    conn: Connection,
}

impl HistoryStore {
    pub fn new(path: PathBuf) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        conn.execute_batch(CREATE_COMMAND_HISTORY_SQL)?;
        Ok(Self { conn })
    }

    pub fn insert(&mut self, record: &CommandExecutionResult) -> AppResult<()> {
        let commands_json = serde_json::to_string(&record.commands)?;
        let duration_ms = i64::try_from(record.duration_ms).unwrap_or(i64::MAX);

        self.conn.execute(
            r#"
            INSERT INTO command_history (
              id, action, project_path, commands_json, success, exit_code,
              stdout, stderr, started_at, finished_at, duration_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                record.id,
                record.action,
                record.project_path,
                commands_json,
                if record.success { 1 } else { 0 },
                record.exit_code,
                record.stdout,
                record.stderr,
                record.started_at,
                record.finished_at,
                duration_ms,
            ],
        )?;

        Ok(())
    }

    pub fn list(&self, limit: u32) -> AppResult<Vec<CommandHistoryRecord>> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT id, action, project_path, commands_json, success, exit_code,
                   stdout, stderr, started_at, finished_at, duration_ms
            FROM command_history
            ORDER BY started_at DESC
            LIMIT ?1
            "#,
        )?;

        let rows = statement.query_map([limit], |row| {
            let commands_json: String = row.get(3)?;
            let commands: Vec<DisplayCommand> = serde_json::from_str(&commands_json)
                .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;
            let duration_ms: i64 = row.get(10)?;

            Ok(CommandHistoryRecord {
                id: row.get(0)?,
                action: row.get(1)?,
                project_path: row.get(2)?,
                commands,
                success: row.get::<_, i64>(4)? == 1,
                exit_code: row.get(5)?,
                stdout: row.get(6)?,
                stderr: row.get(7)?,
                started_at: row.get(8)?,
                finished_at: row.get(9)?,
                duration_ms: u128::try_from(duration_ms).unwrap_or_default(),
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    pub fn clear(&mut self) -> AppResult<()> {
        self.conn.execute("DELETE FROM command_history", [])?;
        Ok(())
    }
}

#[tauri::command]
pub fn list_command_history(
    state: tauri::State<'_, crate::state::AppState>,
    limit: Option<u32>,
) -> AppResult<Vec<CommandHistoryRecord>> {
    let limit = limit.unwrap_or(100).clamp(1, 1000);
    state
        .history
        .lock()
        .map_err(|_| AppError::new("STATE_ERROR", "History store lock is poisoned."))?
        .list(limit)
}

#[tauri::command]
pub fn clear_command_history(state: tauri::State<'_, crate::state::AppState>) -> AppResult<()> {
    state
        .history
        .lock()
        .map_err(|_| AppError::new("STATE_ERROR", "History store lock is poisoned."))?
        .clear()
}
