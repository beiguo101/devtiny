use crate::core::app_error::{AppError, AppResult};
use crate::core::command_runner::{run_command, CommandSpec, DisplayCommand};
use crate::core::path_guard::{
    canonical_project_path, resolve_parent_checked, validate_relative_path,
};
use crate::git::git_status::is_git_repository;
use crate::git::git_status::GitFileStatus;
use crate::history::command_history::CommandExecutionResult;
use crate::state::AppState;
use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHistoryEntry {
    pub commit: String,
    pub short_commit: String,
    pub date: String,
    pub subject: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRevisionContent {
    pub relative_path: String,
    pub commit: String,
    pub content: String,
    pub diff: String,
    pub is_binary: bool,
    pub size: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePointPage {
    pub entries: Vec<FileHistoryEntry>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePointFile {
    pub relative_path: String,
    pub old_relative_path: Option<String>,
    pub status: GitFileStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePointFileDiff {
    pub relative_path: String,
    pub diff: String,
}

#[tauri::command]
pub async fn list_save_points(
    project_path: String,
    skip: usize,
    limit: usize,
) -> AppResult<SavePointPage> {
    let project_path = checked_git_project(&project_path)?;
    if !(1..=50).contains(&limit) {
        return Err(AppError::invalid_input(
            "Save point page size must be between 1 and 50.",
        ));
    }
    let output = run_command(
        &project_path,
        &CommandSpec::new(
            "git",
            vec![
                "log".into(),
                format!("--skip={skip}"),
                format!("--max-count={}", limit + 1),
                "--date=iso-strict".into(),
                "--format=%H%x1f%h%x1f%ad%x1f%s".into(),
            ],
        ),
        30,
    )
    .await?;
    let mut entries = output
        .stdout
        .lines()
        .filter_map(parse_history_line)
        .collect::<Vec<_>>();
    let has_more = entries.len() > limit;
    entries.truncate(limit);
    Ok(SavePointPage { entries, has_more })
}

#[tauri::command]
pub async fn list_save_point_files(
    project_path: String,
    commit: String,
) -> AppResult<Vec<SavePointFile>> {
    let project_path = checked_git_project(&project_path)?;
    let commit = validate_commit(&commit)?;
    let output = std::process::Command::new("git")
        .args([
            "diff-tree",
            "--root",
            "--no-commit-id",
            "--name-status",
            "-r",
            "-z",
            "-M",
            &commit,
        ])
        .current_dir(&project_path)
        .output()?;
    if !output.status.success() {
        return Err(AppError::command_failed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    Ok(parse_save_point_files(&output.stdout))
}

#[tauri::command]
pub async fn get_save_point_file_diff(
    project_path: String,
    relative_path: String,
    commit: String,
) -> AppResult<SavePointFileDiff> {
    let project_path = checked_git_project(&project_path)?;
    let normalized = normalized_existing_or_parent_path(&project_path, &relative_path)?;
    let commit = validate_commit(&commit)?;
    let output = std::process::Command::new("git")
        .args([
            "show",
            "--format=",
            "--find-renames",
            &commit,
            "--",
            &normalized,
        ])
        .current_dir(&project_path)
        .output()?;
    if !output.status.success() {
        return Err(AppError::command_failed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    Ok(SavePointFileDiff {
        relative_path: normalized,
        diff: String::from_utf8_lossy(&output.stdout).to_string(),
    })
}

#[tauri::command]
pub async fn read_file_revision(
    project_path: String,
    relative_path: String,
    commit: String,
) -> AppResult<FileRevisionContent> {
    let project_path = checked_git_project(&project_path)?;
    let normalized = normalized_existing_or_parent_path(&project_path, &relative_path)?;
    let commit = validate_commit(&commit)?;
    let output = std::process::Command::new("git")
        .args(["show", &format!("{commit}:{normalized}")])
        .current_dir(&project_path)
        .output()?;
    if !output.status.success() {
        return Err(AppError::command_failed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    let is_binary = output.stdout.contains(&0);
    let size = output.stdout.len();
    let content = if is_binary {
        String::new()
    } else {
        String::from_utf8_lossy(&output.stdout).to_string()
    };
    let diff_output = std::process::Command::new("git")
        .args([
            "show",
            "--format=",
            "--find-renames",
            &commit,
            "--",
            &normalized,
        ])
        .current_dir(&project_path)
        .output()?;
    let diff = if diff_output.status.success() {
        String::from_utf8_lossy(&diff_output.stdout).to_string()
    } else {
        String::new()
    };
    Ok(FileRevisionContent {
        relative_path: normalized,
        commit,
        content,
        diff,
        is_binary,
        size,
    })
}

#[tauri::command]
pub async fn list_file_history(
    project_path: String,
    relative_path: String,
) -> AppResult<Vec<FileHistoryEntry>> {
    let project_path = checked_git_project(&project_path)?;
    let normalized = normalized_existing_or_parent_path(&project_path, &relative_path)?;

    let output = run_command(
        &project_path,
        &CommandSpec::new(
            "git",
            vec![
                "log".into(),
                "--follow".into(),
                "--date=iso-strict".into(),
                "--format=%H%x1f%h%x1f%ad%x1f%s".into(),
                "--".into(),
                normalized,
            ],
        ),
        30,
    )
    .await?;

    Ok(output
        .stdout
        .lines()
        .filter_map(parse_history_line)
        .collect())
}

#[tauri::command]
pub async fn save_file_revision_as(
    state: State<'_, AppState>,
    project_path: String,
    relative_path: String,
    commit: String,
    target_path: String,
) -> AppResult<()> {
    let project_path = checked_git_project(&project_path)?;
    let normalized = normalized_existing_or_parent_path(&project_path, &relative_path)?;
    let commit = validate_commit(&commit)?;
    let target = validate_target_path(&target_path)?;

    let command = CommandSpec::new("git", vec!["show".into(), format!("{commit}:{normalized}")]);
    let started = Utc::now();
    let output = run_command(&project_path, &command, 30).await?;
    let finished = Utc::now();

    if output.exit_code != Some(0) {
        insert_history(
            &state,
            "git.saveFileRevisionAs",
            &project_path,
            &[command],
            false,
            output.exit_code,
            output.stdout.clone(),
            output.stderr.clone(),
            started,
            finished,
        )?;
        return Err(AppError::command_failed(
            [output.stdout, output.stderr]
                .into_iter()
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(target, output.stdout)?;
    insert_history(
        &state,
        "git.saveFileRevisionAs",
        &project_path,
        &[command],
        true,
        output.exit_code,
        String::new(),
        output.stderr,
        started,
        finished,
    )?;
    Ok(())
}

#[tauri::command]
pub async fn restore_file_revision(
    state: State<'_, AppState>,
    project_path: String,
    relative_path: String,
    commit: String,
) -> AppResult<()> {
    let project_path = checked_git_project(&project_path)?;
    let normalized = normalized_existing_or_parent_path(&project_path, &relative_path)?;
    let commit = validate_commit(&commit)?;

    let command = CommandSpec::new(
        "git",
        vec!["checkout".into(), commit, "--".into(), normalized],
    );
    let started = Utc::now();
    let output = run_command(&project_path, &command, 30).await?;
    let finished = Utc::now();

    if output.exit_code != Some(0) {
        insert_history(
            &state,
            "git.restoreFileRevision",
            &project_path,
            &[command],
            false,
            output.exit_code,
            output.stdout.clone(),
            output.stderr.clone(),
            started,
            finished,
        )?;
        return Err(AppError::command_failed(
            [output.stdout, output.stderr]
                .into_iter()
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }

    insert_history(
        &state,
        "git.restoreFileRevision",
        &project_path,
        &[command],
        true,
        output.exit_code,
        output.stdout,
        output.stderr,
        started,
        finished,
    )?;

    Ok(())
}

fn checked_git_project(project_path: &str) -> AppResult<PathBuf> {
    let project_path = canonical_project_path(project_path)?;
    if !is_git_repository(&project_path) {
        return Err(AppError::invalid_input(
            "Selected project is not a Git repository.",
        ));
    }
    Ok(project_path)
}

fn normalized_existing_or_parent_path(
    project_path: &std::path::Path,
    relative_path: &str,
) -> AppResult<String> {
    let normalized = validate_relative_path(relative_path)?
        .to_string_lossy()
        .replace('\\', "/");
    let _ = resolve_parent_checked(project_path, &normalized)?;
    Ok(normalized)
}

fn validate_commit(commit: &str) -> AppResult<String> {
    let trimmed = commit.trim();
    if (7..=40).contains(&trimmed.len()) && trimmed.chars().all(|ch| ch.is_ascii_hexdigit()) {
        Ok(trimmed.to_string())
    } else {
        Err(AppError::invalid_input("Invalid commit id."))
    }
}

fn validate_target_path(target_path: &str) -> AppResult<PathBuf> {
    let trimmed = target_path.trim();
    if trimmed.is_empty() {
        return Err(AppError::invalid_input("Target path is required."));
    }
    let target = PathBuf::from(trimmed);
    if target.is_absolute() {
        Ok(target)
    } else {
        Err(AppError::invalid_input("Target path must be absolute."))
    }
}

fn parse_history_line(line: &str) -> Option<FileHistoryEntry> {
    let parts = line.split('\u{1f}').collect::<Vec<_>>();
    if parts.len() != 4 {
        return None;
    }
    Some(FileHistoryEntry {
        commit: parts[0].to_string(),
        short_commit: parts[1].to_string(),
        date: parts[2].to_string(),
        subject: parts[3].to_string(),
    })
}

fn parse_save_point_files(output: &[u8]) -> Vec<SavePointFile> {
    let values = output
        .split(|byte| *byte == 0)
        .filter(|value| !value.is_empty())
        .map(|value| String::from_utf8_lossy(value).to_string())
        .collect::<Vec<_>>();
    let mut files = Vec::new();
    let mut index = 0;
    while index < values.len() {
        let code = values[index].as_str();
        index += 1;
        let Some(path) = values.get(index).cloned() else {
            break;
        };
        index += 1;
        let kind = code.chars().next().unwrap_or('M');
        let (relative_path, old_relative_path) = if matches!(kind, 'R' | 'C') {
            let Some(new_path) = values.get(index).cloned() else {
                break;
            };
            index += 1;
            (new_path, Some(path))
        } else {
            (path, None)
        };
        let status = match kind {
            'A' | 'C' => GitFileStatus::Added,
            'D' => GitFileStatus::Deleted,
            'R' => GitFileStatus::Renamed,
            _ => GitFileStatus::Modified,
        };
        files.push(SavePointFile {
            relative_path,
            old_relative_path,
            status,
        });
    }
    files
}

#[allow(clippy::too_many_arguments)]
fn insert_history(
    state: &State<'_, AppState>,
    action: &str,
    project_path: &std::path::Path,
    commands: &[CommandSpec],
    success: bool,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    started: chrono::DateTime<Utc>,
    finished: chrono::DateTime<Utc>,
) -> AppResult<()> {
    let record = CommandExecutionResult {
        id: Uuid::new_v4().to_string(),
        action: action.to_string(),
        project_path: project_path.to_string_lossy().to_string(),
        commands: commands.iter().map(DisplayCommand::from).collect(),
        success,
        exit_code,
        stdout,
        stderr,
        started_at: started.to_rfc3339(),
        finished_at: finished.to_rfc3339(),
        duration_ms: (finished - started).num_milliseconds().max(0) as u128,
    };

    state
        .history
        .lock()
        .map_err(|_| AppError::new("STATE_ERROR", "History store lock is poisoned."))?
        .insert(&record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_file_history_line() {
        let entry = parse_history_line(
            "abcdef0123456789\u{1f}abcdef0\u{1f}2026-06-27T10:00:00+08:00\u{1f}update app",
        )
        .expect("parse history");
        assert_eq!(entry.commit, "abcdef0123456789");
        assert_eq!(entry.short_commit, "abcdef0");
        assert_eq!(entry.subject, "update app");
    }

    #[test]
    fn validates_commit_hash_only() {
        assert!(validate_commit("abcdef0").is_ok());
        assert!(validate_commit("HEAD").is_err());
        assert!(validate_commit("abcdef0 --").is_err());
    }

    #[test]
    fn parses_save_point_files_with_rename() {
        let files = parse_save_point_files(b"M\0src/app.ts\0R100\0old.txt\0new.txt\0");
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].relative_path, "src/app.ts");
        assert_eq!(files[1].old_relative_path.as_deref(), Some("old.txt"));
        assert_eq!(files[1].relative_path, "new.txt");
        assert_eq!(files[1].status, GitFileStatus::Renamed);
    }
}
