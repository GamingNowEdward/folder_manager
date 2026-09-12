import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import type { UnlistenFn } from '@tauri-apps/api/event'
import type { WorkspaceChangedPayload } from '@/infrastructure/tauri/event'

type Handler = (payload: WorkspaceChangedPayload) => void

const handlers: Handler[] = []
const unlistenMock = vi.fn()

vi.mock('@/infrastructure/tauri/event', () => ({
  WORKSPACE_CHANGED_EVENT: 'workspace-changed',
  onWorkspaceChanged: vi.fn(async (handler: Handler) => {
    handlers.push(handler)
    return unlistenMock as UnlistenFn
  })
}))

const loadConfigMock = vi.fn()
vi.mock('@/infrastructure/tauri/config', () => ({
  loadConfig: () => loadConfigMock(),
  scheduleSave: vi.fn()
}))

import { useProjectStore } from '@/features/projects'
import { useStatusStore } from '@/shared/stores/status.store'
import { stopWorkspaceChanges, subscribeWorkspaceChanges } from './useWorkspaceSync'

/** 触发一次 Rust 端广播的「配置已变更」事件，并等待异步 reload 完成。 */
async function emitWorkspaceChanged(payload: WorkspaceChangedPayload): Promise<void> {
  for (const handler of handlers) handler(payload)
  // loadFromDisk → loadConfig（mock promise）→ hydrate → setStatus
  for (let i = 0; i < 5; i += 1) await Promise.resolve()
}

beforeEach(() => {
  stopWorkspaceChanges()
  handlers.length = 0
  unlistenMock.mockClear()
  loadConfigMock.mockReset()
  setActivePinia(createPinia())
  useStatusStore().setStatus('● 就绪', 0)
})

describe('useWorkspaceSync', () => {
  it('reloads the store when the API changes the config', async () => {
    const projectStore = useProjectStore()
    loadConfigMock.mockResolvedValueOnce({
      currentProjectId: 'p1',
      projects: [{ id: 'p1', name: '初始', folders: [] }]
    })
    await projectStore.loadFromDisk()
    expect(projectStore.currentProject?.name).toBe('初始')

    await subscribeWorkspaceChanges()

    // API 侧新增了项目 → Rust 广播事件 → 前端重新拉取权威快照
    loadConfigMock.mockResolvedValueOnce({
      currentProjectId: 'p1',
      projects: [
        { id: 'p1', name: '初始', folders: [] },
        { id: 'p2', name: 'Agent 新增', folders: [{ id: 'f1', name: 'src', path: 'C:\\src' }] }
      ]
    })
    await emitWorkspaceChanged({ revision: 7, source: 'workspace-changed' })

    expect(loadConfigMock).toHaveBeenCalledTimes(2)
    expect(projectStore.projects.map((project) => project.name)).toEqual(['初始', 'Agent 新增'])
    expect(projectStore.projects[1].folders[0].name).toBe('src')
  })

  it('reports the reload in the status bar', async () => {
    loadConfigMock.mockResolvedValue({ currentProjectId: '', projects: [] })
    const statusStore = useStatusStore()

    await subscribeWorkspaceChanges()
    await emitWorkspaceChanged({ revision: 12, source: 'workspace-changed' })

    expect(statusStore.message).toContain('外部更新')
    expect(statusStore.message).toContain('12')
  })

  it('unsubscribes on stop', async () => {
    loadConfigMock.mockResolvedValue({ currentProjectId: '', projects: [] })

    await subscribeWorkspaceChanges()
    stopWorkspaceChanges()

    expect(unlistenMock).toHaveBeenCalledTimes(1)
  })

  it('keeps the UI usable when the event subscription is unavailable', async () => {
    const eventModule = await import('@/infrastructure/tauri/event')
    vi.mocked(eventModule.onWorkspaceChanged).mockRejectedValueOnce(new Error('no tauri'))
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {})

    await expect(subscribeWorkspaceChanges()).resolves.toBeUndefined()

    expect(consoleError).toHaveBeenCalled()
    consoleError.mockRestore()
  })
})
