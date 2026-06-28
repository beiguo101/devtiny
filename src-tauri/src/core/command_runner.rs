use crate::core::app_error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const MAX_OUTPUT_BYTES: usize = 300 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

impl CommandSpec {
    pub fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
        }
    }

    pub fn display(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .map(shell_quote)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayCommand {
    pub program: String,
    pub args: Vec<String>,
    pub display: String,
}

impl From<&CommandSpec> for DisplayCommand {
    fn from(value: &CommandSpec) -> Self {
        Self {
            program: value.program.clone(),
            args: value.args.clone(),
            display: value.display(),
        }
    }
}

#[derive(Debug)]
pub struct SingleCommandOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub async fn run_command(
    project_path: &Path,
    spec: &CommandSpec,
    timeout_seconds: u64,
) -> AppResult<SingleCommandOutput> {
    let program = resolve_program(&spec.program).unwrap_or_else(|| PathBuf::from(&spec.program));
    let mut command = Command::new(program);
    command
        .args(&spec.args)
        .current_dir(project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let output = timeout(Duration::from_secs(timeout_seconds), command.output())
        .await
        .map_err(|_| {
            AppError::command_failed(format!(
                "Command timed out after {timeout_seconds}s: {}",
                spec.display()
            ))
        })??;

    Ok(SingleCommandOutput {
        exit_code: output.status.code(),
        stdout: truncate_output(output.stdout),
        stderr: truncate_output(output.stderr),
    })
}

pub fn command_available(program: &str) -> bool {
    resolve_program(program).is_some()
}

fn resolve_program(program: &str) -> Option<PathBuf> {
    let program_path = Path::new(program);
    if program_path.components().count() > 1 && is_executable_file(program_path) {
        return Some(program_path.to_path_buf());
    }

    command_search_dirs()
        .into_iter()
        .flat_map(|dir| {
            executable_candidates(program)
                .into_iter()
                .map(move |candidate| dir.join(candidate))
        })
        .find(|path| is_executable_file(path))
}

fn command_search_dirs() -> Vec<PathBuf> {
    let mut dirs = std::env::var_os("PATH")
        .map(|path_var| std::env::split_paths(&path_var).collect::<Vec<_>>())
        .unwrap_or_default();

    for dir in fallback_command_dirs() {
        if !dirs.iter().any(|existing| existing == &dir) {
            dirs.push(dir);
        }
    }

    dirs
}

#[cfg(target_os = "macos")]
fn fallback_command_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
        PathBuf::from("/usr/sbin"),
        PathBuf::from("/sbin"),
        PathBuf::from("/Applications/Docker.app/Contents/Resources/bin"),
    ];

    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".docker/bin"));
    }

    dirs
}

#[cfg(target_os = "windows")]
fn fallback_command_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for key in ["ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(root) = std::env::var_os(key) {
            dirs.push(
                PathBuf::from(root)
                    .join("Docker")
                    .join("Docker")
                    .join("resources")
                    .join("bin"),
            );
        }
    }
    dirs
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
fn fallback_command_dirs() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
    ]
}

fn truncate_output(bytes: Vec<u8>) -> String {
    let mut text = String::from_utf8_lossy(&bytes).to_string();
    if text.len() > MAX_OUTPUT_BYTES {
        text.truncate(MAX_OUTPUT_BYTES);
        text.push_str("\n[output truncated]");
    }
    text
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '/' | '-' | '_' | ':' | '='))
    {
        return value.to_string();
    }

    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.is_file()
        && path
            .metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

#[cfg(windows)]
fn executable_candidates(program: &str) -> Vec<String> {
    if Path::new(program).extension().is_some() {
        return vec![program.to_string()];
    }

    let path_ext = std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE;.CMD;.BAT".into());
    path_ext
        .split(';')
        .filter(|ext| !ext.is_empty())
        .map(|ext| format!("{program}{ext}"))
        .collect()
}

#[cfg(not(windows))]
fn executable_candidates(program: &str) -> Vec<String> {
    vec![program.to_string()]
}
