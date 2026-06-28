import { invoke } from '@tauri-apps/api/core'
import type { GitFileStatus } from '../../../core/commands/types'
import type { FileDiff, FileHistoryEntry, GitChangeFile } from './types'

export function listGitChanges(projectPath: string): Promise<GitChangeFile[]> {
  return invoke('list_git_changes', { projectPath })
}

export function getFileDiff(
  projectPath: string,
  relativePath: string,
  status: GitFileStatus
): Promise<FileDiff> {
  return invoke('get_file_diff', { projectPath, relativePath, status })
}

export function listFileHistory(projectPath: string, relativePath: string): Promise<FileHistoryEntry[]> {
  return invoke('list_file_history', { projectPath, relativePath })
}

export function saveFileRevisionAs(
  projectPath: string,
  relativePath: string,
  commit: string,
  targetPath: string
): Promise<void> {
  return invoke('save_file_revision_as', { projectPath, relativePath, commit, targetPath })
}

export function restoreFileRevision(
  projectPath: string,
  relativePath: string,
  commit: string
): Promise<void> {
  return invoke('restore_file_revision', { projectPath, relativePath, commit })
}
