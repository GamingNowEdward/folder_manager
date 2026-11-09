/**
 * 键盘事件的最小结构：让纯函数既能接收真实 KeyboardEvent，也能在单测里用普通对象构造。
 */
export interface KeyLike {
  key: string
  ctrlKey?: boolean
  metaKey?: boolean
  altKey?: boolean
  shiftKey?: boolean
  isComposing?: boolean
  keyCode?: number
}

/** IME 组合中 Chromium 会把 keyCode 置为 229。 */
export const IME_PROCESS_KEY_CODE = 229

/**
 * 输入法组合中的按键不应触发应用快捷键。
 * 例如中文输入法里按 Enter 是「选词」，若当成「确认」会误提交对话框。
 */
export function isImeComposing(event: KeyLike): boolean {
  return event.isComposing === true || event.keyCode === IME_PROCESS_KEY_CODE
}
