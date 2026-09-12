import { computed, reactive } from 'vue'
import type { FolderId } from '@/types'

export type DialogMode = 'add' | 'edit'

interface ProjectDialogState {
  visible: boolean
  mode: DialogMode
}

interface FolderDialogState {
  visible: boolean
  mode: DialogMode
  editingId: FolderId | null
  name: string
  path: string
}

interface ConfirmDialogState {
  visible: boolean
  title: string
  message: string
}

const projectDialog = reactive<ProjectDialogState>({ visible: false, mode: 'add' })
const folderDialog = reactive<FolderDialogState>({
  visible: false,
  mode: 'add',
  editingId: null,
  name: '',
  path: ''
})
const confirmDialog = reactive<ConfirmDialogState>({ visible: false, title: '', message: '' })
let confirmAction: (() => void) | null = null

export function useDialogs() {
  const isAnyDialogOpen = computed(
    () => projectDialog.visible || folderDialog.visible || confirmDialog.visible
  )

  function openProjectDialog(mode: DialogMode): void {
    projectDialog.mode = mode
    projectDialog.visible = true
  }

  function closeProjectDialog(): void {
    projectDialog.visible = false
  }

  function openFolderDialog(
    mode: DialogMode,
    initial: { id?: FolderId; name: string; path: string }
  ): void {
    folderDialog.mode = mode
    folderDialog.editingId = initial.id ?? null
    folderDialog.name = initial.name
    folderDialog.path = initial.path
    folderDialog.visible = true
  }

  function closeFolderDialog(): void {
    folderDialog.visible = false
  }

  function openConfirm(title: string, message: string, onConfirm: () => void): void {
    confirmDialog.title = title
    confirmDialog.message = message
    confirmAction = onConfirm
    confirmDialog.visible = true
  }

  function confirm(): void {
    const action = confirmAction
    confirmAction = null
    confirmDialog.visible = false
    action?.()
  }

  function cancelConfirm(): void {
    confirmAction = null
    confirmDialog.visible = false
  }

  return {
    projectDialog,
    folderDialog,
    confirmDialog,
    isAnyDialogOpen,
    openProjectDialog,
    closeProjectDialog,
    openFolderDialog,
    closeFolderDialog,
    openConfirm,
    confirm,
    cancelConfirm
  }
}
