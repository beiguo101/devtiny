import { invoke } from '@tauri-apps/api/core'
import type { DirectorySummary, FileContent, FileMutationResult, FileTreeNode } from './types'

export function listProjectFiles(projectPath: string): Promise<FileTreeNode[]> {
  return invoke('list_project_files', { projectPath })
}

export function readProjectFile(projectPath: string, relativePath: string): Promise<FileContent> {
  return invoke('read_project_file', { projectPath, relativePath })
}

export function getDirectorySummary(
  projectPath: string,
  relativePath?: string
): Promise<DirectorySummary> {
  return invoke('get_directory_summary', { projectPath, relativePath })
}

export function createProjectFile(
  projectPath: string,
  relativePath: string,
  content = ''
): Promise<FileMutationResult> {
  return invoke('create_project_file', { request: { projectPath, relativePath, content } })
}

export function writeProjectFile(
  projectPath: string,
  relativePath: string,
  content: string
): Promise<FileMutationResult> {
  return invoke('write_project_file', { request: { projectPath, relativePath, content } })
}

export function deleteProjectFile(projectPath: string, relativePath: string): Promise<FileMutationResult> {
  return invoke('delete_project_file', { request: { projectPath, relativePath } })
}
