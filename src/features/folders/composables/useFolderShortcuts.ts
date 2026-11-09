import { onMounted, onUnmounted } from 'vue'
import { useDialogs } from '@/features/dialogs'
import type { FolderSelection } from '@/features/selection'
import { useProjectStore } from '@/features/projects'
import { useStatusStore } from '@/shared/stores/status.store'
import { resolveFolderShortcut } from '../model/shortcut.model'
import { useFolderStore } from '../store/folder.store'

interface FolderShortcutsOptions {
  selection: FolderSelection
  /** 有进行中的指针交互（拖拽 / 框选）时不响应快捷键。 */
  isBusy: () => boolean
}

/**
 * 文件夹卡片区的全部键盘快捷键，共用一个监听器与一套上下文判断。
 */
export function useFolderShortcuts(options: FolderShortcutsOptions): void {
  const projectStore = useProjectStore()
  const folderStore = useFolderStore()
  const dialogs = useDialogs()
  const statusStore = useStatusStore()

  function isTextEditing(): boolean {
    const element = document.activeElement
    if (!element) return false
    if (element.tagName === 'INPUT' || element.tagName === 'TEXTAREA') return true
    if (element.tagName === 'SELECT') return true
    return (element as HTMLElement).isContentEditable === true
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
    const ids = projectStore.currentFolders.map((folder) => folder.id)
    if (ids.length === 0) return
    options.selection.setIds(ids)
    statusStore.setStatus(`已全选 ${ids.length} 个文件夹`)
  }

  function onKeyDown(event: KeyboardEvent): void {
    if (dialogs.isAnyDialogOpen.value || isTextEditing() || options.isBusy()) return
    const shortcut = resolveFolderShortcut(event)
    if (!shortcut) return
    event.preventDefault()
    if (shortcut === 'delete') deleteSelected()
    else selectAll()
  }

  onMounted(() => document.addEventListener('keydown', onKeyDown))
  onUnmounted(() => document.removeEventListener('keydown', onKeyDown))
}
