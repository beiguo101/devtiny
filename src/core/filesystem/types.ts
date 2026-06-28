import type { GitFileStatus } from '../commands/types'

export interface FileTreeNode {
  name: string
  relativePath: string
  kind: 'directory' | 'file'
  status: GitFileStatus
  children: FileTreeNode[]
}

export interface DirectorySummary {
  relativePath: string
  directories: number
  files: number
}

export interface FileContent {
  relativePath: string
  content: string
  isBinary: boolean
  size: number
}

export interface FileMutationResult {
  relativePath: string
}
