use crate::core::app_error::{AppError, AppResult};
use crate::core::command_runner::{command_available, run_command, CommandSpec, DisplayCommand};
use crate::core::path_guard::{
    canonical_project_path, resolve_parent_checked, validate_relative_path,
};
use crate::core::trash::move_path_to_trash;
use crate::git::git_restore::build_restore_steps;
use crate::git::git_status::{is_git_repository, GitFileStatus};
use crate::history::command_history::CommandExecutionResult;
use crate::runtime::compose_actions::build_compose_command;
use crate::state::AppState;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

const PREVIEW_TTL_MINUTES: i64 = 5;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WorkbenchAction {
    #[serde(rename = "git.init")]
    GitInit,
    #[serde(rename = "git.stageFiles")]
    GitStageFiles,
    #[serde(rename = "git.unstageFiles")]
    GitUnstageFiles,
    #[serde(rename = "git.commitFiles")]
    GitCommitFiles,
    #[serde(rename = "git.commitAll")]
    GitCommitAll,
    #[serde(rename = "git.restoreFiles")]
    GitRestoreFiles,
    #[serde(rename = "git.ignoreFiles")]
    GitIgnoreFiles,
    #[serde(rename = "runtime.start")]
    RuntimeStart,
    #[serde(rename = "runtime.stop")]
    RuntimeStop,
    #[serde(rename = "runtime.restart")]
    RuntimeRestart,
    #[serde(rename = "runtime.logs")]
    RuntimeLogs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitFileSelection {
    pub relative_path: String,
    pub status: GitFileStatus,
    pub index_status: Option<String>,
    pub worktree_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchActionPayload {
    pub files: Option<Vec<GitFileSelection>>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewWorkbenchActionRequest {
    pub project_path: String,
    pub action: WorkbenchAction,
    pub payload: Option<WorkbenchActionPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteWorkbenchActionRequest {
    pub preview_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandPreview {
    pub preview_token: String,
    pub project_path: String,
    pub action: WorkbenchAction,
    pub commands: Vec<DisplayCommand>,
    pub affected_files: Vec<GitFileSelection>,
    pub deletes_untracked: bool,
}

#[derive(Debug, Clone)]
pub enum WorkStep {
    Command(CommandSpec),
    TrashPath { relative_path: String },
    AppendGitignore { relative_paths: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct PreviewPlan {
    pub preview_token: String,
    pub project_path: PathBuf,
    pub action: WorkbenchAction,
    pub steps: Vec<WorkStep>,
    pub affected_files: Vec<GitFileSelection>,
    pub deletes_untracked: bool,
    pub created_at: chrono::DateTime<Utc>,
}

#[tauri::command]
pub fn preview_workbench_action(
    state: State<'_, AppState>,
    request: PreviewWorkbenchActionRequest,
) -> AppResult<CommandPreview> {
    let project_path = canonical_project_path(&request.project_path)?;
    let plan = build_preview_plan(project_path, request.action, request.payload)?;
    let preview = plan.to_preview();

    let mut previews = state
        .workbench_previews
        .lock()
        .map_err(|_| AppError::new("STATE_ERROR", "Preview store lock is poisoned."))?;
    let cutoff = Utc::now() - Duration::minutes(PREVIEW_TTL_MINUTES);
    previews.retain(|_, plan| plan.created_at > cutoff);
    previews.insert(plan.preview_token.clone(), plan);

    Ok(preview)
}

#[tauri::command]
pub async fn execute_workbench_action(
    state: State<'_, AppState>,
    request: ExecuteWorkbenchActionRequest,
) -> AppResult<CommandExecutionResult> {
    let plan = {
        let mut previews = state
            .workbench_previews
            .lock()
            .map_err(|_| AppError::new("STATE_ERROR", "Preview store lock is poisoned."))?;
        previews
            .remove(&request.preview_token)
            .ok_or_else(|| AppError::invalid_input("Preview token is invalid or expired."))?
    };

    if Utc::now() - plan.created_at > Duration::minutes(PREVIEW_TTL_MINUTES) {
        return Err(AppError::invalid_input("Preview token is expired."));
    }

    let started = Utc::now();
    let mut stdout_parts = Vec::new();
    let mut stderr_parts = Vec::new();
    let mut exit_code = None;
    let mut success = true;

    for step in &plan.steps {
        match step {
            WorkStep::Command(command) => {
                let output = match run_command(&plan.project_path, command, 60).await {
                    Ok(output) => output,
                    Err(err) => {
                        stderr_parts.push(format!("$ {}\n{}", command.display(), err.message));
                        success = false;
                        break;
                    }
                };
                exit_code = output.exit_code;
                if !output.stdout.is_empty() {
                    stdout_parts.push(format!("$ {}\n{}", command.display(), output.stdout));
                }
                if !output.stderr.is_empty() {
                    stderr_parts.push(format!("$ {}\n{}", command.display(), output.stderr));
                }
                if output.exit_code != Some(0) {
                    success = false;
                    break;
                }
            }
            WorkStep::TrashPath { relative_path } => {
                let path = resolve_parent_checked(&plan.project_path, relative_path)?;
                move_path_to_trash(&path)?;
                stdout_parts.push(format!("move to trash {}", relative_path));
            }
            WorkStep::AppendGitignore { relative_paths } => {
                let gitignore_path = plan.project_path.join(".gitignore");
                let existing = fs::read_to_string(&gitignore_path).unwrap_or_default();
                let mut appended = Vec::new();
                let mut content = existing.clone();
                if !content.is_empty() && !content.ends_with('\n') {
                    content.push('\n');
                }
                for relative_path in relative_paths {
                    let pattern = relative_path.trim();
                    if pattern.is_empty() {
                        continue;
                    }
                    let already_present = existing
                        .lines()
                        .any(|line| line.trim() == pattern || line.trim() == format!("/{pattern}"));
                    if !already_present {
                        content.push_str(pattern);
                        content.push('\n');
                        appended.push(pattern.to_string());
                    }
                }
                fs::write(&gitignore_path, content)?;
                stdout_parts.push(format!("append .gitignore\n{}", appended.join("\n")));
            }
        }
    }

    let finished = Utc::now();
    let record = CommandExecutionResult {
        id: Uuid::new_v4().to_string(),
        action: serde_json::to_value(&plan.action)?
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        project_path: plan.project_path.to_string_lossy().to_string(),
        commands: plan.display_commands(),
        success,
        exit_code,
        stdout: stdout_parts.join("\n"),
        stderr: stderr_parts.join("\n"),
        started_at: started.to_rfc3339(),
        finished_at: finished.to_rfc3339(),
        duration_ms: (finished - started).num_milliseconds().max(0) as u128,
    };

    state
        .history
        .lock()
        .map_err(|_| AppError::new("STATE_ERROR", "History store lock is poisoned."))?
        .insert(&record)?;

    Ok(record)
}

fn build_preview_plan(
    project_path: PathBuf,
    action: WorkbenchAction,
    payload: Option<WorkbenchActionPayload>,
) -> AppResult<PreviewPlan> {
    let mut affected_files =
        normalize_files(payload.as_ref().and_then(|payload| payload.files.clone()))?;
    let mut steps = Vec::new();
    let mut deletes_untracked = false;

    match action {
        WorkbenchAction::GitInit => {
            if !command_available("git") {
                return Err(AppError::unavailable("git is not available."));
            }
            steps.push(WorkStep::Command(CommandSpec::new(
                "git",
                strings(["init"]),
            )));
        }
        WorkbenchAction::GitStageFiles => {
            require_git_repository(&project_path)?;
            let paths = normalized_paths(&project_path, &affected_files)?;
            steps.push(WorkStep::Command(CommandSpec::new(
                "git",
                git_args_with_paths(vec!["add"], &paths),
            )));
        }
        WorkbenchAction::GitUnstageFiles => {
            require_git_repository(&project_path)?;
            for file in &affected_files {
                let path = normalized_path(&project_path, file)?;
                steps.push(WorkStep::Command(CommandSpec::new(
                    "git",
                    unstage_args_for_file(file, &path),
                )));
            }
        }
        WorkbenchAction::GitCommitFiles => {
            require_git_repository(&project_path)?;
            let message = commit_message(payload.as_ref())?;
            let paths = normalized_paths(&project_path, &affected_files)?;
            steps.push(WorkStep::Command(CommandSpec::new(
                "git",
                git_args_with_paths(vec!["add"], &paths),
            )));
            steps.push(WorkStep::Command(CommandSpec::new(
                "git",
                vec!["commit".into(), "-m".into(), message],
            )));
        }
        WorkbenchAction::GitCommitAll => {
            require_git_repository(&project_path)?;
            let message = commit_message(payload.as_ref())?;
            affected_files.clear();
            steps.push(WorkStep::Command(CommandSpec::new(
                "git",
                strings(["add", "."]),
            )));
            steps.push(WorkStep::Command(CommandSpec::new(
                "git",
                vec!["commit".into(), "-m".into(), message],
            )));
        }
        WorkbenchAction::GitRestoreFiles => {
            require_git_repository(&project_path)?;
            let paths = normalized_paths(&project_path, &affected_files)?;
            let status_by_path: HashMap<_, _> = affected_files
                .iter()
                .map(|file| (file.relative_path.clone(), file.status.clone()))
                .collect();
            for path in paths {
                let status = status_by_path
                    .get(&path)
                    .cloned()
                    .unwrap_or(GitFileStatus::Modified);
                if status == GitFileStatus::Untracked {
                    deletes_untracked = true;
                }
                steps.extend(build_restore_steps(&path, &status));
            }
        }
        WorkbenchAction::GitIgnoreFiles => {
            require_git_repository(&project_path)?;
            let paths = normalized_paths(&project_path, &affected_files)?;
            steps.push(WorkStep::AppendGitignore {
                relative_paths: paths,
            });
        }
        WorkbenchAction::RuntimeStart
        | WorkbenchAction::RuntimeStop
        | WorkbenchAction::RuntimeRestart
        | WorkbenchAction::RuntimeLogs => {
            steps.push(WorkStep::Command(build_compose_command(
                &project_path,
                &action,
            )?));
        }
    }

    Ok(PreviewPlan {
        preview_token: Uuid::new_v4().to_string(),
        project_path,
        action,
        steps,
        affected_files,
        deletes_untracked,
        created_at: Utc::now(),
    })
}

impl PreviewPlan {
    pub fn to_preview(&self) -> CommandPreview {
        CommandPreview {
            preview_token: self.preview_token.clone(),
            project_path: self.project_path.to_string_lossy().to_string(),
            action: self.action.clone(),
            commands: self.display_commands(),
            affected_files: self.affected_files.clone(),
            deletes_untracked: self.deletes_untracked,
        }
    }

    pub fn display_commands(&self) -> Vec<DisplayCommand> {
        self.steps
            .iter()
            .map(|step| match step {
                WorkStep::Command(command) => DisplayCommand::from(command),
                WorkStep::TrashPath { relative_path } => DisplayCommand {
                    program: "trash".into(),
                    args: vec![relative_path.clone()],
                    display: format!("move to trash {}", relative_path),
                },
                WorkStep::AppendGitignore { relative_paths } => DisplayCommand {
                    program: "append".into(),
                    args: vec![".gitignore".into()],
                    display: format!("append .gitignore\n{}", relative_paths.join("\n")),
                },
            })
            .collect()
    }
}

fn require_git_repository(project_path: &std::path::Path) -> AppResult<()> {
    if !command_available("git") {
        return Err(AppError::unavailable("git is not available."));
    }
    if is_git_repository(project_path) {
        Ok(())
    } else {
        Err(AppError::invalid_input(
            "Selected project is not a Git repository.",
        ))
    }
}

fn normalize_files(files: Option<Vec<GitFileSelection>>) -> AppResult<Vec<GitFileSelection>> {
    files
        .unwrap_or_default()
        .into_iter()
        .map(|file| {
            let relative_path = validate_relative_path(&file.relative_path)?
                .to_string_lossy()
                .replace('\\', "/");
            Ok(GitFileSelection {
                relative_path,
                status: file.status,
                index_status: file.index_status,
                worktree_status: file.worktree_status,
            })
        })
        .collect()
}

fn normalized_path(project_path: &std::path::Path, file: &GitFileSelection) -> AppResult<String> {
    let relative_path = validate_relative_path(&file.relative_path)?
        .to_string_lossy()
        .replace('\\', "/");
    let _ = resolve_parent_checked(project_path, &relative_path)?;
    Ok(relative_path)
}

fn normalized_paths(
    project_path: &std::path::Path,
    files: &[GitFileSelection],
) -> AppResult<Vec<String>> {
    if files.is_empty() {
        return Err(AppError::invalid_input("At least one file is required."));
    }

    files
        .iter()
        .map(|file| normalized_path(project_path, file))
        .collect()
}

fn unstage_args_for_file(file: &GitFileSelection, path: &str) -> Vec<String> {
    if file.index_status.as_deref() == Some("A") {
        vec!["rm".into(), "--cached".into(), "--".into(), path.into()]
    } else {
        vec![
            "restore".into(),
            "--staged".into(),
            "--".into(),
            path.into(),
        ]
    }
}

fn git_args_with_paths(prefix: Vec<&str>, paths: &[String]) -> Vec<String> {
    prefix
        .into_iter()
        .map(String::from)
        .chain(std::iter::once("--".into()))
        .chain(paths.iter().cloned())
        .collect()
}

fn commit_message(payload: Option<&WorkbenchActionPayload>) -> AppResult<String> {
    let message = payload
        .and_then(|payload| payload.message.as_ref())
        .map(|message| message.trim().to_string())
        .filter(|message| !message.is_empty())
        .ok_or_else(|| AppError::invalid_input("Commit message is required."))?;

    if message.chars().count() > 500 {
        return Err(AppError::invalid_input(
            "Commit message must be 500 characters or fewer.",
        ));
    }
    Ok(message)
}

fn strings<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(String::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn unstage_plan_uses_fixed_restore_staged_command() {
        if !command_available("git") {
            return;
        }

        let project_path = std::env::temp_dir().join(format!("devtiny-{}", Uuid::new_v4()));
        fs::create_dir_all(project_path.join(".git")).expect("create fake git repo");
        fs::create_dir_all(project_path.join("src")).expect("create source dir");
        fs::write(project_path.join("src/main.ts"), "").expect("create source file");

        let plan = build_preview_plan(
            project_path.clone(),
            WorkbenchAction::GitUnstageFiles,
            Some(WorkbenchActionPayload {
                files: Some(vec![GitFileSelection {
                    relative_path: "src/main.ts".into(),
                    status: GitFileStatus::Staged,
                    index_status: Some("M".into()),
                    worktree_status: Some(" ".into()),
                }]),
                message: None,
            }),
        )
        .expect("build unstage plan");

        match &plan.steps[0] {
            WorkStep::Command(command) => {
                assert_eq!(command.program, "git");
                assert_eq!(
                    command.args,
                    vec!["restore", "--staged", "--", "src/main.ts"]
                );
            }
            _ => panic!("expected command step"),
        }

        let _ = fs::remove_dir_all(project_path);
    }

    #[test]
    fn unstage_added_file_uses_rm_cached_for_unborn_head() {
        if !command_available("git") {
            return;
        }

        let project_path = std::env::temp_dir().join(format!("devtiny-{}", Uuid::new_v4()));
        fs::create_dir_all(&project_path).expect("create project");
        fs::create_dir_all(project_path.join(".git")).expect("create fake git repo");
        fs::write(project_path.join("Dockerfile"), "").expect("create file");

        let plan = build_preview_plan(
            project_path.clone(),
            WorkbenchAction::GitUnstageFiles,
            Some(WorkbenchActionPayload {
                files: Some(vec![GitFileSelection {
                    relative_path: "Dockerfile".into(),
                    status: GitFileStatus::Staged,
                    index_status: Some("A".into()),
                    worktree_status: Some(" ".into()),
                }]),
                message: None,
            }),
        )
        .expect("build unstage added plan");

        match &plan.steps[0] {
            WorkStep::Command(command) => {
                assert_eq!(command.program, "git");
                assert_eq!(command.args, vec!["rm", "--cached", "--", "Dockerfile"]);
            }
            _ => panic!("expected command step"),
        }

        let _ = fs::remove_dir_all(project_path);
    }
}
