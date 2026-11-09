import { isImeComposing, type KeyLike } from '@/shared/utils/keyboard'

export type FolderShortcut = 'delete' | 'select-all'

/**
 * 内容区（文件夹卡片）快捷键映射。
 *
 * - Delete：删除选中项
 * - Ctrl+A / Cmd+A：全选当前项目的文件夹
 *
 * 输入法组合中的按键、以及带 Alt / Shift 的组合键不参与。
 */
export function resolveFolderShortcut(event: KeyLike): FolderShortcut | null {
  if (isImeComposing(event)) return null
  if (event.key === 'Delete') return 'delete'
  if (event.altKey === true || event.shiftKey === true) return null
  if (event.ctrlKey === true || event.metaKey === true) {
    if (event.key.toLowerCase() === 'a') return 'select-all'
  }
  return null
}
