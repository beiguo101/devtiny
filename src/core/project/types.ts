export interface ProjectOverview {
  projectPath: string
  gitAvailable: boolean
  isGitRepository: boolean
  hasComposeFile: boolean
  composeFilePath?: string
  running: boolean
  branch?: string
  lastCommit?: string
  lastCommitSubject?: string
  lastCommitAt?: string
}
