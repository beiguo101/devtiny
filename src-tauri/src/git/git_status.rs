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
    pub old_relative_path: Option<String>,
    pub status: GitFileStatus,
    pub index_status: String,
    pub worktree_status: String,
    pub additions: Option<u32>,
    pub deletions: Option<u32>,
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
    pub branch: Option<String>,
    pub last_commit: Option<String>,
    pub last_commit_subject: Option<String>,
    pub last_commit_at: Option<String>,
}

#[tauri::command]
pub async fn get_project_overview(project_path: String) -> AppResult<ProjectOverview> {
    let project_path = canonical_project_path(&project_path)?;
    let git_available = command_available("git");
    let is_git_repository = git_available && is_git_repository(&project_path);
    let branch = git_text(&project_path, &["branch", "--show-current"]);
    let last_commit = git_text(&project_path, &["rev-parse", "--short", "HEAD"]);
    let last_commit_subject = git_text(&project_path, &["log", "-1", "--pretty=%s"]);
    let last_commit_at = git_text(&project_path, &["log", "-1", "--pretty=%cI"]);

    Ok(ProjectOverview {
        project_path: project_path.to_string_lossy().to_string(),
        git_available,
        is_git_repository,
        has_compose_file: false,
        compose_file_path: None,
        running: false,
        branch,
        last_commit,
        last_commit_subject,
        last_commit_at,
    })
}

fn git_text(project_path: &Path, args: &[&str]) -> Option<String> {
    if !is_git_repository(project_path) {
        return None;
    }
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(project_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
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

    let mut changes = parse_status_output(output.stdout.as_bytes());
    enrich_numstat(project_path, &mut changes);
    Ok(changes)
}

fn enrich_numstat(project_path: &Path, changes: &mut [GitChangeFile]) {
    let output = match std::process::Command::new("git")
        .args(["diff", "--numstat", "HEAD", "--"])
        .current_dir(project_path)
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return,
    };
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let mut fields = line.splitn(3, '\t');
        let additions = fields.next().and_then(|value| value.parse::<u32>().ok());
        let deletions = fields.next().and_then(|value| value.parse::<u32>().ok());
        let path = fields.next().unwrap_or_default();
        if let Some(change) = changes
            .iter_mut()
            .find(|change| change.relative_path == path)
        {
            change.additions = additions;
            change.deletions = deletions;
        }
    }
}

pub fn parse_status_output(bytes: &[u8]) -> Vec<GitChangeFile> {
    let mut changes = Vec::new();
    let entries: Vec<_> = bytes
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .collect();
    let mut cursor = 0usize;
    while cursor < entries.len() {
        let entry = entries[cursor];
        if entry.len() < 4 {
            cursor += 1;
            continue;
        }

        let index = entry[0] as char;
        let worktree = entry[1] as char;
        let relative_path = String::from_utf8_lossy(&entry[3..]).to_string();
        let status = classify_status(index, worktree);
        let old_relative_path = if index == 'R' && cursor + 1 < entries.len() {
            cursor += 1;
            Some(String::from_utf8_lossy(entries[cursor]).to_string())
        } else {
            None
        };

        changes.push(GitChangeFile {
            relative_path,
            old_relative_path,
            status,
            index_status: index.to_string(),
            worktree_status: worktree.to_string(),
            additions: None,
            deletions: None,
        });
        cursor += 1;
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

    #[test]
    fn parses_rename_paths_from_zero_format() {
        let changes = parse_status_output(b"R  new-name.txt\0old-name.txt\0");
        assert_eq!(changes[0].relative_path, "new-name.txt");
        assert_eq!(
            changes[0].old_relative_path.as_deref(),
            Some("old-name.txt")
        );
    }
}
