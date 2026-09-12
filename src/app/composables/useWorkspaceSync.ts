import { useProjectStore } from '@/features/projects'
import { onWorkspaceChanged, type WorkspaceChangedPayload } from '@/infrastructure/tauri/event'
import { useStatusStore } from '@/shared/stores/status.store'

/** 外部修改（本地 HTTP API）后刷新界面状态的提示文案。 */
const STATUS_RELOADED = '↻ 配置已被外部更新，已重新加载'

type ProjectStore = ReturnType<typeof useProjectStore>
type StatusStore = ReturnType<typeof useStatusStore>

let unlisten: (() => void) | null = null
let reloading = false
let reloadQueued = false

/**
 * 本地 HTTP API 与 Tauri command 共用同一个 Rust `ConfigService`：
 * 任何一方落盘成功后都会广播 `workspace-changed`，这里据此重新拉取权威快照，
 * 让已打开的 UI 不会停留在旧状态（不再维护第二份长期状态）。
 */
export async function subscribeWorkspaceChanges(): Promise<void> {
  const projectStore = useProjectStore()
  const statusStore = useStatusStore()
  try {
    unlisten = await onWorkspaceChanged((payload: WorkspaceChangedPayload) => {
      void reload(projectStore, statusStore, payload)
    })
  } catch (error) {
    // 事件不可用时 UI 仍可正常使用，只是不会自动刷新外部修改。
    console.error('订阅配置变更事件失败:', error)
  }
}

/**
 * 重新拉取权威快照。
 *
 * `load_config` 返回的永远是**当前**状态而不是事件发生时的快照，所以即使多个事件
 * 交错也不会用旧数据覆盖新数据；这里只做重入合并，避免事件突发时打出一串并发请求。
 */
async function reload(
  projectStore: ProjectStore,
  statusStore: StatusStore,
  payload: WorkspaceChangedPayload
): Promise<void> {
  if (reloading) {
    // 正在刷新：只记一次「还要再刷」，避免并发请求与状态栏抖动
    reloadQueued = true
    return
  }
  reloading = true
  try {
    let loaded = true
    do {
      reloadQueued = false
      loaded = await projectStore.loadFromDisk()
    } while (reloadQueued)
    // 加载失败时不要谎报「已重新加载」
    if (loaded) {
      statusStore.setStatus(`${STATUS_RELOADED}（revision ${payload.revision}）`)
    }
  } finally {
    reloading = false
  }
}

/** 取消订阅（组件卸载 / 测试清理）。 */
export function stopWorkspaceChanges(): void {
  unlisten?.()
  unlisten = null
  reloading = false
  reloadQueued = false
}
