import type { FolderId } from '@/types'
import { useDialogs } from '@/features/dialogs'
import { useProjectStore } from '@/features/projects'
import { copyPath, openFolderInSystem } from '@/infrastructure/tauri/system'
import { useStatusStore } from '@/shared/stores/status.store'
import { toErrorMessage } from '@/shared/utils/errors'
import { findFolderById, isFolderNameTaken } from '../model/folder.model'
import { useFolderStore } from '../store/folder.store'

export interface FolderFormResult {
  name: string
  path: string
}

export function useFolderActions() {
  const projectStore = useProjectStore()
  const folderStore = useFolderStore()
  const dialogs = useDialogs()
  const statusStore = useStatusStore()

  function openAddDialog(): void {
    dialogs.openFolderDialog('add', { name: '', path: '' })
  }

  function openEditDialog(id: FolderId): void {
    const folder = findFolderById(projectStore.currentFolders, id)
    if (!folder) return
    dialogs.openFolderDialog('edit', { id, name: folder.name, path: folder.path })
  }

  function submitForm(result: FolderFormResult): void {
    if (dialogs.folderDialog.mode === 'add') {
      if (isFolderNameTaken(projectStore.currentFolders, result.name)) {
        statusStore.setStatus(`✕ 文件夹「${result.name}」在此项目中已存在`)
        return
      }
      folderStore.addFolder(result.name, result.path)
      statusStore.setStatus(`已添加文件夹「${result.name}」`)
    } else {
      const editingId = dialogs.folderDialog.editingId
      if (!editingId) {
        dialogs.closeFolderDialog()
        return
      }
      if (isFolderNameTaken(projectStore.currentFolders, result.name, editingId)) {
        statusStore.setStatus(`✕ 文件夹「${result.name}」在此项目中已存在`)
        return
      }
      folderStore.updateFolder(editingId, result.name, result.path)
      statusStore.setStatus(`已更新文件夹「${result.name}」`)
    }
    dialogs.closeFolderDialog()
  }

  function openInSystem(path: string): void {
    openFolderInSystem(path)
      .then(() => statusStore.setStatus(`▶ 已打开文件夹：${path}`))
      .catch((error: unknown) => statusStore.setStatus(`✕ ${toErrorMessage(error)}`))
  }

  function copyFolderPath(path: string): void {
    copyPath(path)
      .then(() => statusStore.setStatus(`■ 已复制路径到剪贴板：${path}`))
      .catch(() => statusStore.setStatus('✕ 复制路径失败'))
  }

  function requestDelete(id: FolderId): void {
    const folder = findFolderById(projectStore.currentFolders, id)
    if (!folder) return
    dialogs.openConfirm(
      '删除文件夹',
      `确定要从此项目中移除文件夹「${folder.name}」吗？\n（不会删除实际文件）`,
      () => {
        folderStore.removeFolder(id)
        statusStore.setStatus(`已移除文件夹「${folder.name}」`)
      }
    )
  }

  function requestDeleteSelected(ids: FolderId[], onDeleted?: () => void): void {
    if (ids.length === 0) return
    dialogs.openConfirm(
      '删除文件夹',
      `确定要删除选中的 ${ids.length} 个文件夹吗？\n（不会删除实际文件）`,
      () => {
        folderStore.removeFolders(ids)
        onDeleted?.()
        statusStore.setStatus(`已删除 ${ids.length} 个文件夹`)
      }
    )
  }

  return {
    openAddDialog,
    openEditDialog,
    submitForm,
    openInSystem,
    copyFolderPath,
    requestDelete,
    requestDeleteSelected
  }
}
