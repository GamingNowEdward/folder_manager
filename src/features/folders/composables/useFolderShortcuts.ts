import { nextTick, onMounted, onUnmounted, type Ref } from 'vue'
import type { Folder, FolderId } from '@/types'
import { useDialogs } from '@/features/dialogs'
import type { FolderSelection } from '@/features/selection'
import { useStatusStore } from '@/shared/stores/status.store'
import { findFolderById } from '../model/folder.model'
import {
  resolveEnterAction,
  resolveQuickNavKey,
  resolveQuickNavTarget,
  type EnterAction
} from '../model/quick-nav.model'
import { resolveFolderShortcut } from '../model/shortcut.model'
import { useFolderActions } from './useFolderActions'
import { useFolderStore } from '../store/folder.store'

interface FolderShortcutsOptions {
  selection: FolderSelection
  /** 当前显示顺序：可能已被「字母排序 / 添加日期排序」重排。 */
  folders: Ref<Folder[]>
  /** 滚动容器（`.cards-area`），用于把定位到的卡片带入可视区。 */
  areaRef: Ref<HTMLElement | null>
  /** 有进行中的指针交互（拖拽 / 框选 / 排序菜单）时不响应快捷键。 */
  isBusy: () => boolean
}

/**
 * 文件夹区的全部键盘快捷键，共用一个监听器与一套上下文判断：
 *
 * - 任意字母 / 数字：按显示名首字母定位，连按同一键循环；
 * - Enter：打开当前选中的唯一文件夹；Shift+Enter：复制其路径；
 * - Delete：删除选中项；Ctrl/Cmd+A：全选当前项目的文件夹。
 */
export function useFolderShortcuts(options: FolderShortcutsOptions): void {
  const folderStore = useFolderStore()
  const folderActions = useFolderActions()
  const dialogs = useDialogs()
  const statusStore = useStatusStore()
  let lastKey: string | null = null

  function isTextEditing(): boolean {
    const element = document.activeElement
    if (!element) return false
    if (element.tagName === 'INPUT' || element.tagName === 'TEXTAREA') return true
    if (element.tagName === 'SELECT') return true
    return (element as HTMLElement).isContentEditable === true
  }

  /** 焦点在按钮/链接上时，Enter 属于按钮，不应被「打开文件夹」抢走。 */
  function isButtonFocused(): boolean {
    const element = document.activeElement as HTMLElement | null
    return element?.closest('button, a, [role="button"]') != null
  }

  function selectedFolder(): Folder | null {
    const ids = options.selection.selectedIds.value
    if (ids.length !== 1) return null
    return findFolderById(options.folders.value, ids[0])
  }

  function openSelected(action: EnterAction): boolean {
    const folder = selectedFolder()
    if (!folder) return false
    if (action === 'open') folderActions.openInSystem(folder.path)
    else folderActions.copyFolderPath(folder.path)
    return true
  }

  async function reveal(id: FolderId): Promise<void> {
    await nextTick()
    const element = options.areaRef.value?.querySelector<HTMLElement>(
      `[data-folder-id="${CSS.escape(id)}"]`
    )
    element?.scrollIntoView({ block: 'nearest' })
  }

  function quickNav(key: string): boolean {
    const selected = options.selection.selectedIds.value
    const activeId = selected.length === 1 ? selected[0] : null
    const target = resolveQuickNavTarget(options.folders.value, key, activeId, lastKey === key)
    if (!target) return false

    lastKey = key
    options.selection.selectOne(target.id)
    void reveal(target.id)
    return true
  }

  function deleteSelected(): void {
    const ids = [...options.selection.selectedIds.value]
    if (ids.length === 0) return
    dialogs.openConfirm(
      '删除文件夹',
      `确定要删除选中的 ${ids.length} 个文件夹吗？\n（不会删除实际文件）`,
      () => {
        folderStore.removeFolders(ids)
        options.selection.clear()
        statusStore.setStatus(`已删除 ${ids.length} 个文件夹`)
      }
    )
  }

  function selectAll(): void {
    const ids = options.folders.value.map((folder) => folder.id)
    if (ids.length === 0) return
    options.selection.setIds(ids)
    statusStore.setStatus(`已全选 ${ids.length} 个文件夹`)
  }

  function onKeyDown(event: KeyboardEvent): void {
    if (dialogs.isAnyDialogOpen.value || isTextEditing() || options.isBusy()) return

    const enterAction = resolveEnterAction(event)
    if (enterAction) {
      if (isButtonFocused()) return
      if (!openSelected(enterAction)) return
      event.preventDefault()
      return
    }

    const quickKey = resolveQuickNavKey(event)
    if (quickKey) {
      if (quickNav(quickKey)) event.preventDefault()
      return
    }

    const shortcut = resolveFolderShortcut(event)
    if (!shortcut) return
    event.preventDefault()
    if (shortcut === 'delete') deleteSelected()
    else selectAll()
  }

  onMounted(() => document.addEventListener('keydown', onKeyDown))
  onUnmounted(() => document.removeEventListener('keydown', onKeyDown))
}
