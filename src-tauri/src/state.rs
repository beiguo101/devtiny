use crate::git::git_actions::PreviewPlan;
use crate::history::command_history::HistoryStore;
use crate::runtime::long_task::RuntimeTask;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub history: Mutex<HistoryStore>,
    pub workbench_previews: Mutex<HashMap<String, PreviewPlan>>,
    pub runtime_tasks: Arc<Mutex<HashMap<String, RuntimeTask>>>,
}
