import { onMounted, onUnmounted } from 'vue'
import { useDialogs } from '@/features/dialogs'
import type { FolderSelection } from '@/features/selection'
import { useStatusStore } from '@/shared/stores/status.store'
import { useFolderStore } from '../store/folder.store'

export function useDeleteShortcut(selection: FolderSelection): void {
  const folderStore = useFolderStore()
  const dialogs = useDialogs()
  const statusStore = useStatusStore()

  function onKeyDown(event: KeyboardEvent): void {
    if (document.activeElement?.tagName === 'INPUT') return
    if (event.key !== 'Delete') return
    if (dialogs.isAnyDialogOpen.value) return
    const ids = [...selection.selectedIds.value]
    if (ids.length === 0) return
    event.preventDefault()
    dialogs.openConfirm(
      '删除文件夹',
      `确定要删除选中的 ${ids.length} 个文件夹吗？\n（不会删除实际文件）`,
      () => {
        folderStore.removeFolders(ids)
        selection.clear()
        statusStore.setStatus(`已删除 ${ids.length} 个文件夹`)
      }
    )
  }

  onMounted(() => document.addEventListener('keydown', onKeyDown))
  onUnmounted(() => document.removeEventListener('keydown', onKeyDown))
}
