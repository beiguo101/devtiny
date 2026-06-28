import { invoke } from '@tauri-apps/api/core'
import type {
  ConfigureMirrorsResult,
  CommandExecutionResult,
  CommandPreview,
  PreviewWorkbenchActionRequest,
  RuntimeTask
} from './types'

export function previewWorkbenchAction(request: PreviewWorkbenchActionRequest): Promise<CommandPreview> {
  return invoke('preview_workbench_action', { request })
}

export function executeWorkbenchAction(previewToken: string): Promise<CommandExecutionResult> {
  return invoke('execute_workbench_action', { request: { previewToken } })
}

export function startRuntimeFirstRun(projectPath: string): Promise<RuntimeTask> {
  return invoke('start_runtime_first_run', { request: { projectPath } })
}

export function getRuntimeTask(taskId: string): Promise<RuntimeTask> {
  return invoke('get_runtime_task', { request: { taskId } })
}

export function cancelRuntimeTask(taskId: string): Promise<RuntimeTask> {
  return invoke('cancel_runtime_task', { request: { taskId } })
}

export function configureProjectMirrors(projectPath: string): Promise<ConfigureMirrorsResult> {
  return invoke('configure_project_mirrors', { request: { projectPath } })
}
