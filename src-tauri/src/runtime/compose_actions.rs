use crate::core::app_error::{AppError, AppResult};
use crate::core::command_runner::{command_available, run_command, CommandSpec};
use crate::git::git_actions::WorkbenchAction;
use std::path::{Path, PathBuf};

const COMPOSE_FILES: [&str; 4] = [
    "docker-compose.yml",
    "docker-compose.yaml",
    "compose.yml",
    "compose.yaml",
];

pub fn find_compose_file(project_path: &Path) -> Option<PathBuf> {
    COMPOSE_FILES
        .iter()
        .map(|file| project_path.join(file))
        .find(|path| path.is_file())
}

pub fn build_compose_command(
    project_path: &Path,
    action: &WorkbenchAction,
) -> AppResult<CommandSpec> {
    if !command_available("docker") {
        return Err(AppError::unavailable("docker is not available."));
    }
    if find_compose_file(project_path).is_none() {
        return Err(AppError::invalid_input(
            "No docker-compose.yml, docker-compose.yaml, compose.yml, or compose.yaml file found.",
        ));
    }

    let args = match action {
        WorkbenchAction::RuntimeStart => strings(&["compose", "up", "-d"]),
        WorkbenchAction::RuntimeStop => strings(&["compose", "down"]),
        WorkbenchAction::RuntimeRestart => strings(&["compose", "restart"]),
        WorkbenchAction::RuntimeLogs => strings(&["compose", "logs", "--tail=200"]),
        _ => return Err(AppError::invalid_input("Invalid runtime action.")),
    };
    Ok(CommandSpec::new("docker", args))
}

pub async fn is_compose_running(project_path: &Path) -> bool {
    if !command_available("docker") || find_compose_file(project_path).is_none() {
        return false;
    }

    let output = run_command(project_path, &compose_running_command(), 10).await;

    output
        .map(|output| output.exit_code == Some(0) && !output.stdout.trim().is_empty())
        .unwrap_or(false)
}

fn compose_running_command() -> CommandSpec {
    CommandSpec::new(
        "docker",
        strings(&["compose", "ps", "--status", "running", "-q"]),
    )
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_running_command_is_fixed() {
        let command = compose_running_command();
        assert_eq!(command.program, "docker");
        assert_eq!(
            command.args,
            vec!["compose", "ps", "--status", "running", "-q"]
        );
    }
}
