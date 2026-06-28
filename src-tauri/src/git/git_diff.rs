use crate::core::app_error::AppResult;
use crate::core::command_runner::{run_command, CommandSpec};
use crate::core::path_guard::{
    canonical_project_path, resolve_parent_checked, validate_relative_path,
};
use crate::git::git_status::{is_git_repository, GitFileStatus};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub relative_path: String,
    pub diff: String,
}

#[tauri::command]
pub async fn get_file_diff(
    project_path: String,
    relative_path: String,
    status: GitFileStatus,
) -> AppResult<FileDiff> {
    let project_path = canonical_project_path(&project_path)?;
    if !is_git_repository(&project_path) {
        return Ok(FileDiff {
            relative_path,
            diff: String::new(),
        });
    }

    let relative = validate_relative_path(&relative_path)?;
    let normalized = relative.to_string_lossy().replace('\\', "/");
    let _ = resolve_parent_checked(&project_path, &normalized)?;

    let args = match status {
        GitFileStatus::Staged => vec![
            "diff".to_string(),
            "--cached".to_string(),
            "--".to_string(),
            normalized.clone(),
        ],
        _ => vec!["diff".to_string(), "--".to_string(), normalized.clone()],
    };

    let output = run_command(&project_path, &CommandSpec::new("git", args), 30).await?;
    Ok(FileDiff {
        relative_path: normalized,
        diff: [output.stdout, output.stderr]
            .into_iter()
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
    })
}
