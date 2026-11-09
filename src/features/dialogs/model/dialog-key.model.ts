import { isImeComposing, type KeyLike } from '@/shared/utils/keyboard'

export type DialogKeyAction = 'confirm' | 'cancel' | null

/**
 * 把按键映射为对话框动作。
 *
 * - Enter：确认（对话框里 Enter 一律等同默认按钮）
 * - Escape：取消
 * - Y / N：仅在 `allowQuickKeys` 时生效（确认类对话框；含输入框的对话框必须关闭，否则会干扰打字）
 * - 输入法组合中的按键一律忽略
 */
export function resolveDialogKey(event: KeyLike, allowQuickKeys: boolean): DialogKeyAction {
  if (isImeComposing(event)) return null
  if (event.key === 'Escape') return 'cancel'
  if (event.key === 'Enter') return 'confirm'
  if (!allowQuickKeys) return null
  // 带 Ctrl / Alt / Meta 的组合键留给系统或编辑器，不抢 Y / N。
  if (event.ctrlKey === true || event.altKey === true || event.metaKey === true) return null
  const key = event.key.toLowerCase()
  if (key === 'y') return 'confirm'
  if (key === 'n') return 'cancel'
  return null
}
