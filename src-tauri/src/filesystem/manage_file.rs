use crate::core::app_error::{AppError, AppResult};
use crate::core::path_guard::{
    canonical_project_path, ensure_inside, resolve_parent_checked, validate_relative_path,
};
use crate::core::trash::move_path_to_trash;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMutationRequest {
    pub project_path: String,
    pub relative_path: String,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMutationResult {
    pub relative_path: String,
}

#[tauri::command]
pub fn create_project_file(request: FileMutationRequest) -> AppResult<FileMutationResult> {
    let project_path = canonical_project_path(&request.project_path)?;
    let relative_path = normalize_relative_path(&request.relative_path)?;
    let relative = validate_relative_path(&relative_path)?;
    ensure_existing_ancestors_inside(&project_path, &relative)?;
    let file_path = project_path.join(&relative);

    if file_path.exists() {
        return Err(AppError::invalid_input("File already exists."));
    }

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&file_path, request.content.unwrap_or_default())?;

    Ok(FileMutationResult { relative_path })
}

#[tauri::command]
pub fn write_project_file(request: FileMutationRequest) -> AppResult<FileMutationResult> {
    let project_path = canonical_project_path(&request.project_path)?;
    let relative_path = normalize_relative_path(&request.relative_path)?;
    let file_path = resolve_parent_checked(&project_path, &relative_path)?;

    if !file_path.exists() || !file_path.is_file() {
        return Err(AppError::invalid_input("Selected path is not a file."));
    }

    fs::write(&file_path, request.content.unwrap_or_default())?;
    Ok(FileMutationResult { relative_path })
}

#[tauri::command]
pub fn delete_project_file(request: FileMutationRequest) -> AppResult<FileMutationResult> {
    let project_path = canonical_project_path(&request.project_path)?;
    let relative_path = normalize_relative_path(&request.relative_path)?;
    let file_path = resolve_parent_checked(&project_path, &relative_path)?;

    if !file_path.exists() || !file_path.is_file() {
        return Err(AppError::invalid_input("Selected path is not a file."));
    }

    move_path_to_trash(&file_path)?;
    Ok(FileMutationResult { relative_path })
}

fn normalize_relative_path(relative_path: &str) -> AppResult<String> {
    Ok(validate_relative_path(relative_path)?
        .to_string_lossy()
        .replace('\\', "/"))
}

fn ensure_existing_ancestors_inside(
    project_path: &std::path::Path,
    relative_path: &std::path::Path,
) -> AppResult<()> {
    let mut current = project_path.to_path_buf();
    for component in relative_path.components() {
        current.push(component.as_os_str());
        if current.exists() {
            ensure_inside(project_path, &current.canonicalize()?)?;
        }
    }
    Ok(())
}
