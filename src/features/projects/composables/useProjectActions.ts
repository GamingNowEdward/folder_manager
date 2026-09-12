import { useDialogs } from '@/features/dialogs'
import { useStatusStore } from '@/shared/stores/status.store'
import { isProjectNameTaken } from '../model/project.model'
import { useProjectStore } from '../store/project.store'

export function useProjectActions() {
  const store = useProjectStore()
  const dialogs = useDialogs()
  const statusStore = useStatusStore()

  function openAddDialog(): void {
    dialogs.openProjectDialog('add')
  }

  function openEditDialog(): void {
    if (store.currentProject) dialogs.openProjectDialog('edit')
  }

  function requestDelete(): void {
    const project = store.currentProject
    if (!project) return
    dialogs.openConfirm(
      '删除项目',
      `确定要删除项目「${project.name}」及其所有文件夹配置吗？\n（不会删除实际文件）`,
      () => {
        store.deleteProject(project.id)
        statusStore.setStatus(`已删除项目「${project.name}」`)
      }
    )
  }

  function submitName(name: string): void {
    if (dialogs.projectDialog.mode === 'add') {
      if (isProjectNameTaken(store.projects, name)) {
        statusStore.setStatus(`✕ 项目「${name}」已存在`)
        return
      }
      store.addProject(name)
      statusStore.setStatus(`已创建项目「${name}」`)
    } else {
      const project = store.currentProject
      if (!project) {
        dialogs.closeProjectDialog()
        return
      }
      if (isProjectNameTaken(store.projects, name, project.id)) {
        statusStore.setStatus(`✕ 项目「${name}」已存在`)
        return
      }
      store.renameProjectById(project.id, name)
      statusStore.setStatus(`已重命名为「${name}」`)
    }
    dialogs.closeProjectDialog()
  }

  return { openAddDialog, openEditDialog, requestDelete, submitName }
}
