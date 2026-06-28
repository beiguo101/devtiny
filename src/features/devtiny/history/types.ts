import type { DisplayCommand } from '../../../core/commands/types'

export interface CommandHistoryRecord {
  id: string
  action: string
  projectPath: string
  commands: DisplayCommand[]
  success: boolean
  exitCode: number | null
  stdout: string
  stderr: string
  startedAt: string
  finishedAt: string
  durationMs: number
}
