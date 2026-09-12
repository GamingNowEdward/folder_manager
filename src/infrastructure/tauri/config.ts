import type { WorkspaceData } from '@/types'
import { SAVE_DEBOUNCE_MS } from '@/shared/constants'
import { COMMANDS, invokeCommand } from './commands'
import { toConfigDto, toWorkspace, type ConfigDto } from './config.mapper'

let saveTimer: ReturnType<typeof setTimeout> | null = null
let pendingWorkspace: WorkspaceData | null = null

export async function loadConfig(): Promise<WorkspaceData> {
  const dto = await invokeCommand<ConfigDto>(COMMANDS.loadConfig)
  return toWorkspace(dto)
}

export function scheduleSave(workspace: WorkspaceData): void {
  pendingWorkspace = workspace
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    saveTimer = null
    void flushSave()
  }, SAVE_DEBOUNCE_MS)
}

export async function flushSave(): Promise<void> {
  const workspace = pendingWorkspace
  pendingWorkspace = null
  if (!workspace) return
  try {
    await invokeCommand<void>(COMMANDS.saveConfig, { data: toConfigDto(workspace) })
  } catch (error) {
    console.error('配置保存失败:', error)
  }
}
