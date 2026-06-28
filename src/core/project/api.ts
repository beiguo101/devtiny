import { invoke } from '@tauri-apps/api/core'
import type { ProjectOverview } from './types'

export function getProjectOverview(projectPath: string): Promise<ProjectOverview> {
  return invoke('get_project_overview', { projectPath })
}
