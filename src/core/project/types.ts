export interface ProjectOverview {
  projectPath: string
  gitAvailable: boolean
  isGitRepository: boolean
  hasComposeFile: boolean
  composeFilePath?: string
  running: boolean
}
