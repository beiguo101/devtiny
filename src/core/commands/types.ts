export type GitFileStatus =
  | 'clean'
  | 'modified'
  | 'added'
  | 'deleted'
  | 'untracked'
  | 'staged'
  | 'renamed'
  | 'conflicted'

export type WorkbenchAction =
  | 'git.init'
  | 'git.stageFiles'
  | 'git.unstageFiles'
  | 'git.commitFiles'
  | 'git.commitAll'
  | 'git.restoreFiles'
  | 'git.ignoreFiles'
  | 'runtime.start'
  | 'runtime.stop'
  | 'runtime.restart'
  | 'runtime.logs'

export type RuntimeTaskState = 'running' | 'succeeded' | 'failed' | 'cancelled'

export interface GitFileSelection {
  relativePath: string
  status: GitFileStatus
  indexStatus?: string
  worktreeStatus?: string
}

export interface WorkbenchActionPayload {
  files?: GitFileSelection[]
  message?: string
}

export interface PreviewWorkbenchActionRequest {
  projectPath: string
  action: WorkbenchAction
  payload?: WorkbenchActionPayload
}

export interface DisplayCommand {
  program: string
  args: string[]
  display: string
}

export interface CommandPreview {
  previewToken: string
  projectPath: string
  action: WorkbenchAction
  commands: DisplayCommand[]
  affectedFiles: GitFileSelection[]
  deletesUntracked: boolean
}

export interface CommandExecutionResult {
  id: string
  action: string
  projectPath: string
  commands: DisplayCommand[]
  success: boolean
  exitCode: number | null
  stdout: string
  stderr: string
  startedAt: string
  finishedAt: string
  durationMs: number
}

export interface RuntimeTask {
  taskId: string
  projectPath: string
  action: string
  command: string
  status: RuntimeTaskState
  output: string
  exitCode: number | null
  startedAt: string
  finishedAt?: string
}

export interface ConfigureMirrorsResult {
  files: string[]
  dockerfilePath: string
  message: string
}
