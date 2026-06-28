use crate::core::app_error::{AppError, AppResult};
use crate::core::path_guard::{canonical_project_path, resolve_existing_path};
use serde::Serialize;
use std::fs;

const MAX_READ_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContent {
    pub relative_path: String,
    pub content: String,
    pub is_binary: bool,
    pub size: u64,
}

#[tauri::command]
pub fn read_project_file(project_path: String, relative_path: String) -> AppResult<FileContent> {
    let project_path = canonical_project_path(&project_path)?;
    let file_path = resolve_existing_path(&project_path, &relative_path)?;
    if !file_path.is_file() {
        return Err(AppError::invalid_input("Selected path is not a file."));
    }

    let metadata = fs::metadata(&file_path)?;
    if metadata.len() > MAX_READ_BYTES {
        return Err(AppError::invalid_input(
            "File is larger than the 1MB preview limit.",
        ));
    }

    let bytes = fs::read(&file_path)?;
    let is_binary = bytes.contains(&0);
    let content = if is_binary {
        String::new()
    } else {
        String::from_utf8_lossy(&bytes).to_string()
    };

    Ok(FileContent {
        relative_path,
        content,
        is_binary,
        size: metadata.len(),
    })
}
