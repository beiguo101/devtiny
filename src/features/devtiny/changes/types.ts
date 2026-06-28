import type { GitFileStatus } from '../../../core/commands/types'

export interface GitChangeFile {
  relativePath: string
  status: GitFileStatus
  indexStatus: string
  worktreeStatus: string
}

export interface FileDiff {
  relativePath: string
  diff: string
}

export interface FileHistoryEntry {
  commit: string
  shortCommit: string
  date: string
  subject: string
}
