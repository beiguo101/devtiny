import { invoke } from '@tauri-apps/api/core'
import type { CommandHistoryRecord } from './types'

export function listCommandHistory(limit = 100): Promise<CommandHistoryRecord[]> {
  return invoke('list_command_history', { limit })
}

export function clearCommandHistory(): Promise<void> {
  return invoke('clear_command_history')
}
