use crate::core::app_error::{AppError, AppResult};
use crate::core::command_runner::{command_available, CommandSpec};
use crate::core::path_guard::canonical_project_path;
use crate::runtime::compose_actions::find_compose_file;
use crate::state::AppState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tauri::State;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

const MAX_TASK_OUTPUT_BYTES: usize = 500 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeTask {
    pub task_id: String,
    pub project_path: String,
    pub action: String,
    pub command: String,
    pub status: RuntimeTaskState,
    pub output: String,
    pub exit_code: Option<i32>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeTaskState {
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRuntimeTaskRequest {
    pub project_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRuntimeTaskRequest {
    pub task_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelRuntimeTaskRequest {
    pub task_id: String,
}

#[tauri::command]
pub fn start_runtime_first_run(
    state: State<'_, AppState>,
    request: StartRuntimeTaskRequest,
) -> AppResult<RuntimeTask> {
    if !command_available("docker") {
        return Err(AppError::unavailable("docker is not available."));
    }

    let project_path = canonical_project_path(&request.project_path)?;
    if find_compose_file(&project_path).is_none() {
        return Err(AppError::invalid_input(
            "No docker-compose.yml, docker-compose.yaml, compose.yml, or compose.yaml file found.",
        ));
    }

    let spec = CommandSpec::new(
        "docker",
        vec!["compose".into(), "up".into(), "--build".into(), "-d".into()],
    );
    let command_display = spec.display();
    let task_id = Uuid::new_v4().to_string();
    let task = RuntimeTask {
        task_id: task_id.clone(),
        project_path: project_path.to_string_lossy().to_string(),
        action: "runtime.firstRun".into(),
        command: command_display.clone(),
        status: RuntimeTaskState::Running,
        output: format!("$ {}\n", command_display),
        exit_code: None,
        started_at: Utc::now().to_rfc3339(),
        finished_at: None,
    };

    state
        .runtime_tasks
        .lock()
        .map_err(|_| AppError::new("STATE_ERROR", "Runtime task store lock is poisoned."))?
        .insert(task_id.clone(), task.clone());

    let tasks = state.inner().runtime_tasks.clone();
    std::thread::spawn(move || {
        let output = Arc::new(Mutex::new(format!("$ {}\n", command_display)));
        let output_for_runtime = output.clone();
        let finished_task_id = task_id.clone();
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
        {
            Ok(runtime) => runtime,
            Err(err) => {
                finish_task(
                    &tasks,
                    &finished_task_id,
                    RuntimeTaskState::Failed,
                    None,
                    &format!("Failed to start task runtime: {err}"),
                );
                return;
            }
        };

        runtime.block_on(async move {
            let mut command = Command::new(&spec.program);
            command
                .args(&spec.args)
                .current_dir(&project_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);

            let mut child = match command.spawn() {
                Ok(child) => child,
                Err(err) => {
                    finish_task(
                        &tasks,
                        &finished_task_id,
                        RuntimeTaskState::Failed,
                        None,
                        &format!("Failed to start command: {err}"),
                    );
                    return;
                }
            };

            if let Some(stdout) = child.stdout.take() {
                let output = output_for_runtime.clone();
                tokio::spawn(async move {
                    read_lines(stdout, "", output).await;
                });
            }
            if let Some(stderr) = child.stderr.take() {
                let output = output_for_runtime.clone();
                tokio::spawn(async move {
                    read_lines(stderr, "ERR ", output).await;
                });
            }

            loop {
                if task_was_cancelled(&tasks, &finished_task_id) {
                    let _ = child.kill().await;
                    append_cancelled_output(&tasks, &finished_task_id);
                    break;
                }

                if let Ok(buffer) = output_for_runtime.lock() {
                    if !buffer.is_empty() {
                        append_task_output(&tasks, &finished_task_id, &buffer);
                    }
                }

                match child.try_wait() {
                    Ok(Some(status)) => {
                        if let Ok(buffer) = output_for_runtime.lock() {
                            if !buffer.is_empty() {
                                append_task_output(&tasks, &finished_task_id, &buffer);
                            }
                        }
                        let state = if status.success() {
                            RuntimeTaskState::Succeeded
                        } else {
                            RuntimeTaskState::Failed
                        };
                        finish_task(&tasks, &finished_task_id, state, status.code(), "");
                        break;
                    }
                    Ok(None) => {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                    Err(err) => {
                        finish_task(
                            &tasks,
                            &finished_task_id,
                            RuntimeTaskState::Failed,
                            None,
                            &format!("Failed to poll command: {err}"),
                        );
                        break;
                    }
                }
            }
        });
    });

    Ok(task)
}

#[tauri::command]
pub fn get_runtime_task(
    state: State<'_, AppState>,
    request: GetRuntimeTaskRequest,
) -> AppResult<RuntimeTask> {
    state
        .runtime_tasks
        .lock()
        .map_err(|_| AppError::new("STATE_ERROR", "Runtime task store lock is poisoned."))?
        .get(&request.task_id)
        .cloned()
        .ok_or_else(|| AppError::invalid_input("Runtime task was not found."))
}

#[tauri::command]
pub fn cancel_runtime_task(
    state: State<'_, AppState>,
    request: CancelRuntimeTaskRequest,
) -> AppResult<RuntimeTask> {
    let mut tasks = state
        .runtime_tasks
        .lock()
        .map_err(|_| AppError::new("STATE_ERROR", "Runtime task store lock is poisoned."))?;
    let task = tasks
        .get_mut(&request.task_id)
        .ok_or_else(|| AppError::invalid_input("Runtime task was not found."))?;

    if task.status == RuntimeTaskState::Running {
        task.status = RuntimeTaskState::Cancelled;
        task.finished_at = Some(Utc::now().to_rfc3339());
        task.output
            .push_str("\n[DevTiny] 已请求停止首次依赖任务，正在终止后台命令...\n");
        truncate_string(&mut task.output);
    }

    Ok(task.clone())
}

async fn read_lines<R>(reader: R, prefix: &'static str, output: Arc<Mutex<String>>)
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(mut buffer) = output.lock() {
            buffer.push_str(prefix);
            buffer.push_str(&line);
            buffer.push('\n');
            truncate_string(&mut buffer);
        }
    }
}

fn append_task_output(
    tasks: &Mutex<std::collections::HashMap<String, RuntimeTask>>,
    task_id: &str,
    output: &str,
) {
    if output.is_empty() {
        return;
    }
    let Ok(mut tasks) = tasks.lock() else {
        return;
    };
    let Some(task) = tasks.get_mut(task_id) else {
        return;
    };
    if task.status != RuntimeTaskState::Running {
        return;
    }
    task.output = output.to_string();
    truncate_string(&mut task.output);
}

fn task_was_cancelled(
    tasks: &Mutex<std::collections::HashMap<String, RuntimeTask>>,
    task_id: &str,
) -> bool {
    let Ok(tasks) = tasks.lock() else {
        return false;
    };
    tasks
        .get(task_id)
        .map(|task| task.status == RuntimeTaskState::Cancelled)
        .unwrap_or(false)
}

fn append_cancelled_output(
    tasks: &Mutex<std::collections::HashMap<String, RuntimeTask>>,
    task_id: &str,
) {
    let Ok(mut tasks) = tasks.lock() else {
        return;
    };
    let Some(task) = tasks.get_mut(task_id) else {
        return;
    };
    task.output.push_str("[DevTiny] 后台命令已停止。\n");
    task.exit_code = None;
    task.finished_at = Some(Utc::now().to_rfc3339());
    truncate_string(&mut task.output);
}

fn finish_task(
    tasks: &Mutex<std::collections::HashMap<String, RuntimeTask>>,
    task_id: &str,
    status: RuntimeTaskState,
    exit_code: Option<i32>,
    message: &str,
) {
    let Ok(mut tasks) = tasks.lock() else {
        return;
    };
    let Some(task) = tasks.get_mut(task_id) else {
        return;
    };
    if !message.is_empty() {
        task.output.push_str(message);
        task.output.push('\n');
    }
    truncate_string(&mut task.output);
    task.status = status;
    task.exit_code = exit_code;
    task.finished_at = Some(Utc::now().to_rfc3339());
}

fn truncate_string(value: &mut String) {
    if value.len() <= MAX_TASK_OUTPUT_BYTES {
        return;
    }
    let keep_from = value.len() - MAX_TASK_OUTPUT_BYTES;
    value.replace_range(..keep_from, "[output truncated]\n");
}
