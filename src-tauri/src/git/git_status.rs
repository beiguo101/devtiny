use crate::core::app_error::AppResult;
use crate::core::command_runner::{command_available, run_command, CommandSpec};
use crate::core::path_guard::canonical_project_path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GitFileStatus {
    Clean,
    Modified,
    Added,
    Deleted,
    Untracked,
    Staged,
    Renamed,
    Conflicted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitChangeFile {
    pub relative_path: String,
    pub status: GitFileStatus,
    pub index_status: String,
    pub worktree_status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectOverview {
    pub project_path: String,
    pub git_available: bool,
    pub is_git_repository: bool,
    pub has_compose_file: bool,
    pub compose_file_path: Option<String>,
    pub running: bool,
}

#[tauri::command]
pub async fn get_project_overview(project_path: String) -> AppResult<ProjectOverview> {
    let project_path = canonical_project_path(&project_path)?;
    let git_available = command_available("git");
    let is_git_repository = git_available && is_git_repository(&project_path);
    let compose_file = crate::runtime::compose_actions::find_compose_file(&project_path);
    let running = crate::runtime::compose_actions::is_compose_running(&project_path).await;

    Ok(ProjectOverview {
        project_path: project_path.to_string_lossy().to_string(),
        git_available,
        is_git_repository,
        has_compose_file: compose_file.is_some(),
        compose_file_path: compose_file.map(|path| path.to_string_lossy().to_string()),
        running,
    })
}

#[tauri::command]
pub async fn list_git_changes(project_path: String) -> AppResult<Vec<GitChangeFile>> {
    let project_path = canonical_project_path(&project_path)?;
    list_git_changes_for_project(&project_path).await
}

pub fn is_git_repository(project_path: &Path) -> bool {
    project_path
        .ancestors()
        .any(|path| path.join(".git").exists())
}

pub fn collect_git_status_map(project_path: &Path) -> AppResult<HashMap<String, GitFileStatus>> {
    if !is_git_repository(project_path) || !command_available("git") {
        return Ok(HashMap::new());
    }

    let output = std::process::Command::new("git")
        .args(["status", "--porcelain=v1", "-z", "-uall"])
        .current_dir(project_path)
        .output()?;

    if !output.status.success() {
        return Ok(HashMap::new());
    }

    let changes = parse_status_output(&output.stdout);
    Ok(changes
        .into_iter()
        .map(|change| (change.relative_path, change.status))
        .collect())
}

pub async fn list_git_changes_for_project(project_path: &Path) -> AppResult<Vec<GitChangeFile>> {
    if !is_git_repository(project_path) {
        return Ok(Vec::new());
    }

    let output = run_command(
        project_path,
        &CommandSpec::new("git", strings(["status", "--porcelain=v1", "-z", "-uall"])),
        30,
    )
    .await?;

    Ok(parse_status_output(output.stdout.as_bytes()))
}

pub fn parse_status_output(bytes: &[u8]) -> Vec<GitChangeFile> {
    let mut changes = Vec::new();
    let entries = bytes
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty());
    for entry in entries {
        if entry.len() < 4 {
            continue;
        }

        let index = entry[0] as char;
        let worktree = entry[1] as char;
        let path = String::from_utf8_lossy(&entry[3..]).to_string();
        let relative_path = path.split(" -> ").last().unwrap_or(&path).to_string();
        let status = classify_status(index, worktree);

        changes.push(GitChangeFile {
            relative_path,
            status,
            index_status: index.to_string(),
            worktree_status: worktree.to_string(),
        });
    }

    changes
}

pub fn classify_status(index: char, worktree: char) -> GitFileStatus {
    if index == '?' && worktree == '?' {
        return GitFileStatus::Untracked;
    }
    if index == 'U' || worktree == 'U' {
        return GitFileStatus::Conflicted;
    }
    if index == 'R' {
        return GitFileStatus::Renamed;
    }
    if index != ' ' && index != '?' {
        return GitFileStatus::Staged;
    }
    match worktree {
        'M' => GitFileStatus::Modified,
        'A' => GitFileStatus::Added,
        'D' => GitFileStatus::Deleted,
        _ => GitFileStatus::Clean,
    }
}

fn strings<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(String::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status_output() {
        let changes =
            parse_status_output(b" M src/main.ts\0?? new.txt\0A  staged.ts\0 D old.txt\0");
        assert_eq!(changes[0].status, GitFileStatus::Modified);
        assert_eq!(changes[1].status, GitFileStatus::Untracked);
        assert_eq!(changes[2].status, GitFileStatus::Staged);
        assert_eq!(changes[3].status, GitFileStatus::Deleted);
    }
}
