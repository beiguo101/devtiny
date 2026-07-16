import { invoke } from '@tauri-apps/api/core'
import type { GitFileStatus } from '../../../core/commands/types'
import type { FileDiff, FileHistoryEntry, FileRevisionContent, GitChangeFile, IgnoredRule, SavePointFile, SavePointPage } from './types'

export function listGitChanges(projectPath: string): Promise<GitChangeFile[]> {
  return invoke('list_git_changes', { projectPath })
}

export function listIgnoredRules(projectPath: string): Promise<IgnoredRule[]> {
  return invoke('list_ignored_rules', { projectPath })
}

export function addIgnoredRule(projectPath: string, relativePath: string, mode: 'file' | 'extension'): Promise<IgnoredRule> {
  return invoke('add_ignored_rule', { projectPath, relativePath, mode })
}

export function removeIgnoredRule(projectPath: string, rule: string): Promise<void> {
  return invoke('remove_ignored_rule', { projectPath, rule })
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

export function listSavePoints(projectPath: string, skip = 0, limit = 3): Promise<SavePointPage> {
  return invoke('list_save_points', { projectPath, skip, limit })
}

export function listSavePointFiles(projectPath: string, commit: string): Promise<SavePointFile[]> {
  return invoke('list_save_point_files', { projectPath, commit })
}

export function getSavePointFileDiff(projectPath: string, relativePath: string, commit: string): Promise<FileDiff> {
  return invoke('get_save_point_file_diff', { projectPath, relativePath, commit })
}

export function readFileRevision(projectPath: string, relativePath: string, commit: string): Promise<FileRevisionContent> {
  return invoke('read_file_revision', { projectPath, relativePath, commit })
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
