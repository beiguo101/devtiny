use crate::core::app_error::AppResult;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn app_data_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app.path().app_data_dir()?;
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn history_db_path(app: &AppHandle) -> AppResult<PathBuf> {
    Ok(app_data_dir(app)?.join("command-history.sqlite3"))
}
