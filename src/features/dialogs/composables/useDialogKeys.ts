import { onMounted, onUnmounted } from 'vue'
import { resolveDialogKey } from '../model/dialog-key.model'

interface DialogKeysOptions {
  onConfirm: () => void
  onCancel: () => void
  /** 是否响应 Y / N（确认类对话框开启；含输入框的对话框保持关闭）。 */
  allowQuickKeys?: boolean
  /** 确认动作当前是否可用，例如保存中或表单不完整时按 Enter 不提交。 */
  canConfirm?: () => boolean
}

/**
 * 对话框级键盘绑定：Enter 确认、Escape 取消、可选 Y / N。
 *
 * 用捕获阶段监听 document，而不是绑在面板上——这样焦点落在遮罩、正文或按钮上都有效。
 * preventDefault + stopPropagation 同时做两件事：阻止聚焦按钮被 Enter 再合成一次 click
 *（否则确认会执行两次），以及阻止事件继续冒泡到全局快捷键。
 */
export function useDialogKeys(options: DialogKeysOptions): void {
  function onKeyDown(event: KeyboardEvent): void {
    const action = resolveDialogKey(event, options.allowQuickKeys === true)
    if (!action) return
    if (action === 'confirm' && options.canConfirm && !options.canConfirm()) return
    event.preventDefault()
    event.stopPropagation()
    if (action === 'confirm') options.onConfirm()
    else options.onCancel()
  }

  onMounted(() => document.addEventListener('keydown', onKeyDown, true))
  onUnmounted(() => document.removeEventListener('keydown', onKeyDown, true))
}
