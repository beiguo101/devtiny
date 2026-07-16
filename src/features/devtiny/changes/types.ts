import type { GitFileStatus } from '../../../core/commands/types'

export interface GitChangeFile {
  relativePath: string
  status: GitFileStatus
  indexStatus: string
  worktreeStatus: string
  oldRelativePath?: string
  additions?: number
  deletions?: number
}

export interface FileDiff {
  relativePath: string
  diff: string
}

export interface IgnoredRule {
  rule: string
  displayName: string
  mode: 'file' | 'extension'
}

export interface FileHistoryEntry {
  commit: string
  shortCommit: string
  date: string
  subject: string
}

export interface SavePointPage {
  entries: FileHistoryEntry[]
  hasMore: boolean
}

export interface SavePointFile {
  relativePath: string
  oldRelativePath?: string
  status: GitFileStatus
}

export interface FileRevisionContent {
  relativePath: string
  commit: string
  content: string
  diff: string
  isBinary: boolean
  size: number
}
