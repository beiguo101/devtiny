export type WorkbenchView = 'overview' | 'changes' | 'files'

export type ChangeStatus = 'clean' | 'modified' | 'added' | 'deleted' | 'renamed'

export type DiffLineKind = 'context' | 'added' | 'deleted' | 'hunk'

export interface DiffLine {
  kind: DiffLineKind
  oldLine?: number
  newLine?: number
  content: string
}

export interface MockProjectFile {
  path: string
  name: string
  status: ChangeStatus
  oldPath?: string
  size: string
  modifiedAt: string
  language: string
  content: string
  binary?: boolean
  previewUnavailable?: string
  additions: number
  deletions: number
  hasSelectedChanges: boolean
  hasUnselectedChanges: boolean
  indexStatus?: string
  worktreeStatus?: string
  ignoreRule?: string
  diff: DiffLine[]
}

export interface FileTreeEntry {
  id: string
  name: string
  path: string
  kind: 'directory' | 'file'
  file?: MockProjectFile
  children: FileTreeEntry[]
}

export interface SaveResult {
  message: string
  fileCount: number
  additions: number
  deletions: number
  files: string[]
  remainingCount: number
}

export type ScenarioId =
  | 'normal'
  | 'clean'
  | 'non-git'
  | 'empty-git'
  | 'partial'
  | 'added'
  | 'deleted'
  | 'renamed'
  | 'binary'
  | 'large-diff'
  | 'commit-success'
  | 'commit-failure'
  | 'undo-confirm'
  | 'undo-success'
  | 'operation-failure'
  | 'loading'
  | 'read-failure'

export interface ScenarioOption {
  id: ScenarioId
  label: string
}
