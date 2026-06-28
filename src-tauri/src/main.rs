#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod filesystem;
mod git;
mod history;
mod runtime;
mod state;
mod storage;

use core::app_paths::history_db_path;
use filesystem::list_files::{get_directory_summary, list_project_files};
use filesystem::manage_file::{create_project_file, delete_project_file, write_project_file};
use filesystem::read_file::read_project_file;
use git::git_actions::{execute_workbench_action, preview_workbench_action};
use git::git_diff::get_file_diff;
use git::git_file_history::{list_file_history, restore_file_revision, save_file_revision_as};
use git::git_status::{get_project_overview, list_git_changes};
use history::command_history::{clear_command_history, list_command_history, HistoryStore};
use state::AppState;
use std::sync::{Arc, Mutex};
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let history = HistoryStore::new(history_db_path(app.handle())?)?;

            app.manage(AppState {
                history: Mutex::new(history),
                workbench_previews: Mutex::new(Default::default()),
                runtime_tasks: Arc::new(Mutex::new(Default::default())),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_project_overview,
            list_project_files,
            get_directory_summary,
            read_project_file,
            create_project_file,
            write_project_file,
            delete_project_file,
            list_git_changes,
            get_file_diff,
            list_file_history,
            save_file_revision_as,
            restore_file_revision,
            preview_workbench_action,
            execute_workbench_action,
            runtime::long_task::start_runtime_first_run,
            runtime::long_task::get_runtime_task,
            runtime::long_task::cancel_runtime_task,
            runtime::project_setup::configure_project_mirrors,
            list_command_history,
            clear_command_history
        ])
        .run(tauri::generate_context!())
        .expect("failed to run DevTiny");
}
