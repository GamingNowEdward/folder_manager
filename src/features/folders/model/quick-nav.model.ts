import type { Folder, FolderId } from '@/types'
import { isImeComposing, type KeyLike } from '@/shared/utils/keyboard'

export type EnterAction = 'open' | 'copy'

const QUICK_NAV_KEY = /^[a-z0-9]$/i

/**
 * 首字母定位的触发键：单个 ASCII 字母或数字，且不带 Ctrl / Alt / Meta。
 *
 * - `Shift` 允许（`Shift+R` 仍按 `r` 匹配，避免大写输入失效）。
 * - 输入法组合中的按键、中文、符号、功能键一律忽略。
 */
export function resolveQuickNavKey(event: KeyLike): string | null {
  if (isImeComposing(event)) return null
  if (event.ctrlKey === true || event.altKey === true || event.metaKey === true) return null
  if (event.key.length !== 1 || !QUICK_NAV_KEY.test(event.key)) return null
  return event.key.toLowerCase()
}

/** 显示名（备注名）首字符；空名返回空串。**不读取 path**。 */
export function folderInitial(folder: Folder): string {
  const trimmed = folder.name.trim()
  return trimmed ? trimmed.charAt(0).toLowerCase() : ''
}

/** 按显示名首字符筛选，保持传入顺序（即当前显示顺序）。 */
export function findFoldersByInitial(folders: Folder[], key: string): Folder[] {
  const needle = key.toLowerCase()
  return folders.filter((folder) => folderInitial(folder) === needle)
}

/**
 * 计算本次定位目标：
 * - 首次按键（`repeat=false`）或当前项不在候选中 → 候选第一个；
 * - 连续按同一个键（`repeat=true`）→ 当前项在候选中的下一个，末尾回绕；
 * - 无候选 → `null`（调用方应保持「无反应」）。
 */
export function resolveQuickNavTarget(
  folders: Folder[],
  key: string,
  activeId: FolderId | null,
  repeat: boolean
): Folder | null {
  const matches = findFoldersByInitial(folders, key)
  if (matches.length === 0) return null
  if (!repeat || !activeId) return matches[0]

  const currentIndex = matches.findIndex((folder) => folder.id === activeId)
  if (currentIndex < 0) return matches[0]
  return matches[(currentIndex + 1) % matches.length]
}

/** `Enter` 打开；`Shift+Enter` 复制路径；其他组合不参与。 */
export function resolveEnterAction(event: KeyLike): EnterAction | null {
  if (isImeComposing(event)) return null
  if (event.key !== 'Enter') return null
  if (event.ctrlKey === true || event.altKey === true || event.metaKey === true) return null
  return event.shiftKey === true ? 'copy' : 'open'
}
