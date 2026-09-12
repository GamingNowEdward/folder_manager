import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/**
 * Rust 端（Tauri command 与本地 HTTP API 共用的 ConfigService）在配置落盘后广播的事件。
 * 事件名必须与 `src-tauri/src/application/config_service.rs` 的
 * `WORKSPACE_CHANGED_EVENT` 保持一致。
 */
export const WORKSPACE_CHANGED_EVENT = 'workspace-changed'

export interface WorkspaceChangedPayload {
  revision: number
  source: string
}

/**
 * 订阅「配置已被外部修改」事件。返回取消订阅函数。
 * 只做事件转发，不读文件、不做业务判断 —— 刷新由 store 的 loadFromDisk 完成。
 */
export async function onWorkspaceChanged(
  handler: (payload: WorkspaceChangedPayload) => void
): Promise<UnlistenFn> {
  return listen<WorkspaceChangedPayload>(WORKSPACE_CHANGED_EVENT, (event) => {
    handler(event.payload)
  })
}
