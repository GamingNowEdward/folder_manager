import { useProjectStore } from '@/features/projects'
import { onWorkspaceChanged, type WorkspaceChangedPayload } from '@/infrastructure/tauri/event'
import { useStatusStore } from '@/shared/stores/status.store'

/** 外部修改（本地 HTTP API）后刷新界面状态的提示文案。 */
const STATUS_RELOADED = '↻ 配置已被外部更新，已重新加载'

let unlisten: (() => void) | null = null

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
      void projectStore.loadFromDisk().then(() => {
        statusStore.setStatus(`${STATUS_RELOADED}（revision ${payload.revision}）`)
      })
    })
  } catch (error) {
    // 事件不可用时 UI 仍可正常使用，只是不会自动刷新外部修改。
    console.error('订阅配置变更事件失败:', error)
  }
}

/** 取消订阅（组件卸载 / 测试清理）。 */
export function stopWorkspaceChanges(): void {
  unlisten?.()
  unlisten = null
}
